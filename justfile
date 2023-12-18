#!/usr/bin/env just --justfile

extension_file := './target/debug/examples/libsqlite_compressions'

@_default:
    just --list --unsorted

# Clean all build artifacts
clean:
    cargo clean

build: build-lib build-ext

build-lib:
    cargo build --workspace --all-targets --bins --tests --lib --benches

build-ext *ARGS:
    cargo build --example sqlite_compressions --no-default-features --features loadable_extension,gzip,brotli {{ ARGS }}

# Run cargo fmt and cargo clippy
lint: fmt clippy

# Run cargo fmt
fmt:
    cargo +nightly fmt -- --config imports_granularity=Module,group_imports=StdExternalCrate

# Run cargo clippy
clippy:
    cargo clippy -- -D warnings
    cargo clippy --workspace --all-targets --bins --tests --lib --benches --examples -- -D warnings

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Test documentation
test-doc:
    cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Run all tests
test:
    rustc --version
    cargo --version
    cargo fmt --all -- --check
    RUSTFLAGS='-D warnings' cargo build
    {{ just_executable() }} test-lib
    @echo "### DOCS #######################################################################################################################"
    {{ just_executable() }} test-doc
    @echo "### CLIPPY #####################################################################################################################"
    {{ just_executable() }} clippy
    @echo "### BUILD EXTENSION ############################################################################################################"
    {{ just_executable() }} build-ext
    @echo "### TEST EXTENSION #############################################################################################################"
    {{ just_executable() }} test-ext

# Test the library
test-lib *ARGS: \
    ( test-one-lib "--no-default-features" "--features" "trace,gzip"        ) \
    ( test-one-lib "--no-default-features" "--features" "trace,brotli"      ) \
    ( test-one-lib "--no-default-features" "--features" "gzip,brotli"       ) \
    ( test-one-lib "--no-default-features" "--features" "trace,gzip,brotli" )

test-ext: \
    is-sqlite3-available \
    ( test-one-ext "SELECT hex(gzip('12345'));"                 "1F8B08000000000000FF333432363105001C3AF5CB05000000" ) \
    ( test-one-ext "SELECT gzip_decode(gzip('12345'));"         "12345" ) \
    ( test-one-ext "SELECT gzip_decode(gzip('12345', 1));"      "12345" ) \
    ( test-one-ext "SELECT gzip_decode(gzip('12345', 9));"      "12345" ) \
    ( test-one-ext "SELECT gzip_test(gzip('12345'));"           "1"     ) \
    ( test-one-ext "SELECT hex(brotli('12345'));"               "0B0280313233343503" ) \
    ( test-one-ext "SELECT brotli_decode(brotli('12345'));"     "12345" ) \
    ( test-one-ext "SELECT brotli_decode(brotli('12345', 1));"  "12345" ) \
    ( test-one-ext "SELECT brotli_decode(brotli('12345', 9));"  "12345" ) \
    ( test-one-ext "SELECT gzip_test(gzip('12345'));"           "1"     )

[private]
test-one-lib *ARGS:
    @echo "### TEST {{ ARGS }} #######################################################################################################################"
    cargo test {{ ARGS }}

[private]
is-sqlite3-available:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Check if sqlite3 executable exists"
    if ! command -v sqlite3 &> /dev/null; then
        echo "sqlite3 executable could not be found"
        exit 1
    fi
    sqlite3 --version

[private]
test-one-ext SQL EXPECTED:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ ! -f {{ quote(extension_file) }} ]]; then
        echo "Extension file {{ quote(extension_file) }} does not exist. Run 'just build-ext' first."
        exit 1
    fi
    echo "Expecting '{{ EXPECTED }}'  from  {{ SQL }}"

    RESULT=$(sqlite3 <<EOF
    .log stderr
    .load {{ quote(extension_file) }}
    {{ SQL }}
    EOF
    )
    if [ "$RESULT" != "{{ EXPECTED }}" ]; then
        echo "Failed SQL: $sql"
        echo "Expected:   $expected"
        echo "Actual:     $actual"
        exit 1
    fi

# Run integration tests and save its output as the new expected output
bless *ARGS: (cargo-install "insta" "cargo-insta")
    cargo insta test --accept --unreferenced=auto {{ ARGS }}

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD="" *ARGS="":
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v $COMMAND &> /dev/null; then
        echo "$COMMAND could not be found. Installing it with    cargo install ${INSTALL_CMD:-$COMMAND} {{ ARGS }}"
        cargo install ${INSTALL_CMD:-$COMMAND} {{ ARGS }}
    fi
