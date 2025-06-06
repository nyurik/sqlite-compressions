#!/usr/bin/env just --justfile

main_crate := 'sqlite-compressions'
features_flag := '--all-features'
bin_name := snakecase(main_crate)
sqlite3 := 'sqlite3'

# if running in CI, treat warnings as errors by setting RUSTFLAGS and RUSTDOCFLAGS to '-D warnings' unless they are already set
# Use `CI=true just ci-test` to run the same tests as in GitHub CI.
# Use `just env-info` to see the current values of RUSTFLAGS and RUSTDOCFLAGS
ci_mode := if env('CI', '') != '' {'1'} else {''}
export RUSTFLAGS := env('RUSTFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUSTDOCFLAGS := env('RUSTDOCFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUST_BACKTRACE := env('RUST_BACKTRACE', if ci_mode == '1' {'1'} else {''})

@_default:
    {{just_executable()}} --list

# Run benchmarks
bench:
    cargo bench
    open target/criterion/report/index.html

# Run integration tests and save its output as the new expected output
bless *args:  (cargo-install 'cargo-insta')
    cargo insta test --accept --unreferenced=delete {{args}}

# Build the project
build: build-lib build-ext

# Build extension binary
build-ext *args:
    cargo build --example {{bin_name}} --no-default-features --features default_loadable_extension {{args}}

# Build the lib
build-lib:
    cargo build --workspace

# Quick compile without building a binary
check:
    cargo check --workspace --all-targets {{features_flag}}

# Quick compile - lib-only
check-lib:
    cargo check --workspace

# Generate code coverage report to upload to codecov.io
ci-coverage: env-info && \
            (coverage '--codecov --output-path target/llvm-cov/codecov.info')
    # ATTENTION: the full file path above is used in the CI workflow
    mkdir -p target/llvm-cov

# Run all tests as expected by CI
ci-test: env-info test-fmt check clippy test test-ext test-doc && assert-git-is-clean

# Run minimal subset of tests to ensure compatibility with MSRV
ci-test-msrv: env-info check-lib test

# Clean all build artifacts
clean:
    cargo clean

# Run cargo clippy to lint the code
clippy *args:
    cargo clippy --workspace --all-targets {{features_flag}} {{args}}
    cargo clippy --no-default-features --features default_loadable_extension {{args}}

# Generate code coverage report. Will install `cargo llvm-cov` if missing.
coverage *args='--no-clean --open':  (cargo-install 'cargo-llvm-cov')
    # do not enable --all-features here as it will cause sqlite runtime errors
    cargo llvm-cov --workspace --all-targets --include-build-script {{args}}
    # TODO: add test coverage for the loadable extension too, and combine them
    # cargo llvm-cov --example {{bin_name}} --no-default-features --features default_loadable_extension --codecov --output-path codecov.info

cross-build-ext *args:
    cross build --example {{bin_name}} --no-default-features --features default_loadable_extension {{args}}

cross-build-ext-aarch64:  (cross-build-ext '--target=aarch64-unknown-linux-gnu' '--release')

cross-test-ext-aarch64:
    docker run \
            --rm \
            -v "$(pwd):/workspace" \
            -w /workspace \
            --entrypoint sh \
            -e EXTENSION_FILE=target/aarch64-unknown-linux-gnu/release/examples/lib{{bin_name}} \
            --platform linux/arm64 \
            arm64v8/ubuntu \
            -c 'apt-get update && apt-get install -y sqlite3 && tests/test-ext.sh'

# Build and open code documentation
docs *args='--open':
    DOCS_RS=1 cargo doc --no-deps {{args}} --workspace {{features_flag}}

# Print environment info
env-info:
    @echo "Running {{if ci_mode == '1' {'in CI mode'} else {'in dev mode'} }} on {{os()}} / {{arch()}}"
    {{just_executable()}} --version
    rustc --version
    cargo --version
    rustup --version
    @echo "RUSTFLAGS='$RUSTFLAGS'"
    @echo "RUSTDOCFLAGS='$RUSTDOCFLAGS'"
    @echo "RUST_BACKTRACE='$RUST_BACKTRACE'"

# Reformat all code `cargo fmt`. If nightly is available, use it for better results
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if rustup component list --toolchain nightly | grep rustfmt &> /dev/null; then
        echo 'Reformatting Rust code using nightly Rust fmt to sort imports'
        cargo +nightly fmt --all -- --config imports_granularity=Module,group_imports=StdExternalCrate
    else
        echo 'Reformatting Rust with the stable cargo fmt.  Install nightly with `rustup install nightly` for better results'
        cargo fmt --all
    fi

# Get any package's field from the metadata
get-crate-field field package=main_crate:  (assert-cmd 'jq')
    cargo metadata --format-version 1 | jq -e -r '.packages | map(select(.name == "{{package}}")) | first | .{{field}} | select(. != null)'

# Get the minimum supported Rust version (MSRV) for the crate
get-msrv package=main_crate:  (get-crate-field 'rust_version' package)

# Find the minimum supported Rust version (MSRV) using cargo-msrv extension, and update Cargo.toml
msrv:  (cargo-install 'cargo-msrv')
    cargo msrv find --write-msrv --ignore-lockfile

release *args='':  (cargo-install 'release-plz')
    release-plz {{args}}

# Check semver compatibility with prior published version. Install it with `cargo install cargo-semver-checks`
semver *args:  (cargo-install 'cargo-semver-checks')
    cargo semver-checks {{features_flag}} {{args}}

# Switch to the minimum rusqlite version
set-min-rusqlite-version:
    #!/usr/bin/env bash
    set -euo pipefail
    MIN_RUSQL_VER="$(grep '^rusqlite =.*version = ">=' Cargo.toml | sed -E 's/.*version = "[^"0-9]*([0-9.-]+).*/\1/')"
    echo "Switching to minimum rusqlite version: $MIN_RUSQL_VER"
    cargo update -p rusqlite --precise "$MIN_RUSQL_VER"

# Run all unit and integration tests
test: \
        ( test-one-lib ) \
        ( test-one-lib '--no-default-features' '--features' 'gzip,brotli,bzip2,bsdiff4,bsdiffraw' ) \
        ( test-one-lib '--no-default-features' '--features' 'trace,brotli'    ) \
        ( test-one-lib '--no-default-features' '--features' 'trace,bsdiff4'   ) \
        ( test-one-lib '--no-default-features' '--features' 'trace,bsdiffraw' ) \
        ( test-one-lib '--no-default-features' '--features' 'trace,bzip2'     ) \
        ( test-one-lib '--no-default-features' '--features' 'trace,gzip'      )
    cargo test --doc  # do not enable --all-features here as it will cause sqlite runtime errors

# Test documentation generation
test-doc:  (docs '')

# Test extension by loading it into sqlite and running SQL tests
test-ext: build-ext
    ./tests/test-ext.sh

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

# Find unused dependencies. Install it with `cargo install cargo-udeps`
udeps:  (cargo-install 'cargo-udeps')
    cargo +nightly udeps --workspace --all-targets {{features_flag}}

# Update all dependencies, including breaking changes. Requires nightly toolchain (install with `rustup install nightly`)
update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

# Ensure that a certain command is available
[private]
assert-cmd command:
    @if ! type {{command}} > /dev/null; then \
        echo "Command '{{command}}' could not be found. Please make sure it has been installed on your computer." ;\
        exit 1 ;\
    fi

# Make sure the git repo has no uncommitted changes
[private]
assert-git-is-clean:
    @if [ -n "$(git status --untracked-files --porcelain)" ]; then \
      >&2 echo "ERROR: git repo is no longer clean. Make sure compilation and tests artifacts are in the .gitignore, and no repo files are modified." ;\
      >&2 echo "######### git status ##########" ;\
      git status ;\
      git --no-pager diff ;\
      exit 1 ;\
    fi

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD='' *args='':
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v $COMMAND > /dev/null; then
        if ! command -v cargo-binstall > /dev/null; then
            echo "$COMMAND could not be found. Installing it with    cargo install ${INSTALL_CMD:-$COMMAND} --locked {{args}}"
            cargo install ${INSTALL_CMD:-$COMMAND} --locked {{args}}
        else
            echo "$COMMAND could not be found. Installing it with    cargo binstall ${INSTALL_CMD:-$COMMAND} --locked {{args}}"
            cargo binstall ${INSTALL_CMD:-$COMMAND} --locked {{args}}
        fi
    fi

[private]
is-sqlite3-available:
    if ! command -v {{sqlite3}} > /dev/null; then \
        echo "{{sqlite3}} executable could not be found" ;\
        exit 1 ;\
    fi
    echo "Found {{sqlite3}} executable:"
    {{sqlite3}} --version

[private]
test-one-lib *args:
    @echo "### TEST {{args}} #######################################################################################################################"
    cargo test {{args}}
