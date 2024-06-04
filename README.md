# sqlite-compressions

[![GitHub](https://img.shields.io/badge/github-sqlite--compressions-8da0cb?logo=github)](https://github.com/nyurik/sqlite-compressions)
[![crates.io version](https://img.shields.io/crates/v/sqlite-compressions.svg)](https://crates.io/crates/sqlite-compressions)
[![docs.rs docs](https://docs.rs/sqlite-compressions/badge.svg)](https://docs.rs/sqlite-compressions)
[![crates.io version](https://img.shields.io/crates/l/sqlite-compressions.svg)](https://github.com/nyurik/sqlite-compressions/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/nyurik/sqlite-compressions/actions/workflows/ci.yml/badge.svg)](https://github.com/nyurik/sqlite-compressions/actions)

Implement SQLite compression, decompression, and testing functions for Brotli, bzip2, and gzip encodings, as well as
[bsdiff4](https://github.com/mendsley/bsdiff#readme) and [raw bsdiff](https://github.com/space-wizards/bsdiff-rs#readme)
binary diffing and patching support.
Functions are available as a loadable extension, or as a Rust library.

See also [SQLite-hashes](https://github.com/nyurik/sqlite-hashes) extension for MD5, SHA1, SHA224, SHA256, SHA384,
SHA512, FNV1a, xxHash hashing functions.

## Usage

This SQLite extension adds functions for brotli, bzip2, and gzip compressions like `gzip(data, [quality])`,
decoding `gzip_decode(data)`, and testing `gzip_test(data)` functions. Both encoding and decoding functions return
blobs, and the
testing function returns a true/false. The encoding functions can encode text and blob values, but will raise an error
on other types like integers and floating point numbers. All functions will return `NULL` if the input data is `NULL`.

`bsdiff4(source, target)` will return a binary diff between two blobs, and `bspatch4(source, diff)` will apply the diff
to the source blob to produce the target blob. The diff and patch functions will raise an error if the input data is not
blobs or if the diff is invalid. If either input is `NULL`, the diff and patch functions will return `NULL`.

Similar `bsdiffraw(source, target)` and `bspatchraw(source, diff)` functions are available for raw bsdiff format. Raw
format is not compressed and does not have any magic number prefix. If the internal format provided
by [bsdiff crate](https://github.com/space-wizards/bsdiff-rs#readme) changes, we will add a separate function for it.

### Extension

To use as an extension, load the `libsqlite_compressions.so` shared library into SQLite.

```bash
$ sqlite3
sqlite> .load ./libsqlite_compressions
sqlite> SELECT hex(brotli('Hello world!'));
8B058048656C6C6F20776F726C642103
sqlite> SELECT brotli_decode(x'8B058048656C6C6F20776F726C642103');
Hello world!
sqlite> SELECT brotli_test(x'8B058048656C6C6F20776F726C642103');
1
```

### Rust library

To use as a Rust library, add `sqlite-compressions` to your `Cargo.toml` dependencies. Then, register the needed
functions with `register_compression_functions(&db)`. This will register all available functions, or you can
use `register_gzip_functions(&db)`, `register_brotli_functions(&db)`, `register_bzip2_functions(&db)` to register just
the needed ones (you may also
disable the default features to reduce compile time and binary size).

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

    // Test that diffing source and target blobs can be applied to source to get target.
    let sql = "SELECT bspatch4('source', bsdiff4('source', 'target'));";
    let res: Vec<u8> = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    assert_eq!(res, b"target");

    // Test that diffing source and target blobs can be applied
    // to source to get target when using raw bsdiff format.
    let sql = "SELECT bspatchraw('source', bsdiffraw('source', 'target'));";
    let res: Vec<u8> = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    assert_eq!(res, b"target");
}
```

#### Using with SQLx

To use with [SQLx](https://crates.io/crates/sqlx), you need to get the raw handle from the `SqliteConnection` and pass it to the registration function.

```rust,ignore
use sqlx::sqlite::SqliteConnection;

async fn register_functions(sqlx_conn: &SqliteConnection) {
    // SAFETY: No query must be performed on `sqlx_conn` until `handle_lock` is dropped.
    let mut handle_lock = sqlx_conn.lock_handle().await.unwrap();
    let handle = handle_lock.as_raw_handle().as_ptr();

    // SAFETY: this is safe as long as handle_lock is valid.
    let rusqlite_conn = unsafe { Connection::from_handle(handle) }.unwrap();

    // Registration is attached to the connection, not to rusqlite_conn,
    // so it will be available for the entire lifetime of the `sqlx_conn`.
    // Registration will be automatically dropped when SqliteConnection is dropped.
    register_compression_functions(&rusqlite_conn).unwrap();
}
```

## Crate features

By default, this crate will compile with all features. You can enable just the ones you need to reduce compile time and
binary size.

```toml
[dependencies]
sqlite-compressions = { version = "0.2", default-features = false, features = ["brotli"] }
``` 

* **trace** - enable tracing support, logging all function calls and their arguments
* **brotli** - enable Brotli compression support
* **bzip2** - enable bzip2 compression support
* **gzip** - enable GZIP compression support
* **bsdiff4** - enable bsdiff4 binary diffing and patching support
* **bsdiffraw** - enable bsdiff binary diffing and patching support using raw format

The **loadable_extension** feature should only be used when building a `.so` / `.dylib` / `.dll` extension file that can
be loaded directly into sqlite3 executable.

## Development

* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`.
  Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.
* On `git push`, it will run a few validations, including `cargo fmt`, `cargo clippy`, and `cargo test`.
  Use `git push --no-verify` to skip these checks.

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
