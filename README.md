# sqlite-compressions

[![GitHub](https://img.shields.io/badge/github-sqlite--compressions-8da0cb?logo=github)](https://github.com/nyurik/sqlite-compressions)
[![crates.io version](https://img.shields.io/crates/v/sqlite-compressions.svg)](https://crates.io/crates/sqlite-compressions)
[![docs.rs docs](https://docs.rs/sqlite-compressions/badge.svg)](https://docs.rs/sqlite-compressions)
[![crates.io version](https://img.shields.io/crates/l/sqlite-compressions.svg)](https://github.com/nyurik/sqlite-compressions/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/nyurik/sqlite-compressions/actions/workflows/ci.yml/badge.svg)](https://github.com/nyurik/sqlite-compressions/actions)


Implement SQLite compression, decompression, and testing functions for Brotli and gzip encodings. Functions are available as a loadable extension, or as a Rust library.

This crate uses [rusqlite](https://crates.io/crates/rusqlite) to add user-defined functions using static linking.

## Usage

For each compression name, this crate provides encoding `<...>(data, [quality])`, decoding `<...>_decode(data)`, and testing `<...>_test(data)` functions. For example, for `GZIP` it would create `gzip`, `gzip_decode`, and `gzip_test`. Both encoding and decoding return blobs, and the testing function returns a boolean.  The encoding functions can encode text and blob values, but will raise an error on other types like integers and floating point numbers. All functions will return `NULL` if the input data is `NULL`.

### Extension
To use as an extension, load the `sqlite_compressions.so` shared library into SQLite (works with `gzip` and `brotli`).

```bash
$ sqlite3
sqlite> .load sqlite_compressions.so
sqlite> SELECT hex(brotli('Hello world!'));
8B058048656C6C6F20776F726C642103
sqlite> SELECT brotli_decode(x'8B058048656C6C6F20776F726C642103');
Hello world!
sqlite> SELECT brotli_test(x'8B058048656C6C6F20776F726C642103');
1
```

### Rust library
To use as a Rust library, add `sqlite-compressions` to your `Cargo.toml` dependencies.  Then, register the needed functions with `register_compression_functions(&db)`.  This will register all available functions, or you can use `register_gzip_functions(&db)` or `register_brotli_functions(&db)` to register just the needed ones (you may also disable the default features to reduce compile time and binary size).

```rust
use sqlite_compressions::{register_compression_functions, rusqlite::Connection};

fn main() {
    // Connect to SQLite DB and register needed functions
    let db = Connection::open_in_memory().unwrap();
    // can also use encoding-specific ones like register_gzip_functions(&db)  
    register_compression_functions(&db).unwrap();

    // Encode 'password' using GZIP, and dump resulting BLOB as a HEX string
    let sql = "SELECT hex(gzip('password'));";
    let res: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    assert_eq!(res, "1F8B08000000000000FF2B482C2E2ECF2F4A0100D546C23508000000");

    // Encode 'password' using Brotli, decode it, and convert the blob to text
    let sql = "SELECT CAST(brotli_decode(brotli('password')) AS TEXT);";
    let res: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    assert_eq!(res, "password");

    // Test that Brotli-encoded value is correct.
    let sql = "SELECT brotli_test(brotli('password'));";
    let res: bool = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    assert!(res);
}
```

## Crate features
By default, this crate will compile with all features. You can enable just the ones you need to reduce compile time and binary size.

```toml
[dependencies]
sqlite-compressions = { version = "0.2", default-features = false, features = ["brotli"] }
``` 

* **trace** - enable tracing support, logging all function calls and their arguments
* **brotli** - enable Brotli compression support
* **gzip** - enable GZIP compression support

The **loadable_extension** feature should only be used when building a `.so` or `.dll` extension file that can be loaded directly into sqlite3 executable.

## Development
* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`. Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.
* On `git push`, it will run a few validations, including `cargo fmt`, `cargo clippy`, and `cargo test`.  Use `git push --no-verify` to skip these checks.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
