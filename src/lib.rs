#![cfg_attr(feature = "default", doc = include_str!("../README.md"))]
//
// Unsafe code is required for cdylib, so only use it for this crate
#![forbid(unsafe_code)]

#[cfg(not(any(feature = "gzip", feature = "brotli",)))]
compile_error!("At least one of the features `gzip`, or `brotli` must be enabled.");

/// Re-export of the [`rusqlite`](https://crates.io/crates/rusqlite) crate to avoid version conflicts.
pub use rusqlite;

use crate::rusqlite::{Connection, Result};

mod common;

#[cfg(feature = "gzip")]
mod gzip;
#[cfg(feature = "gzip")]
pub use crate::gzip::register_gzip_functions;

#[cfg(feature = "brotli")]
mod brotli;
#[cfg(feature = "brotli")]
pub use crate::brotli::register_brotli_functions;

/// Register all compression functions for the given `SQLite` connection.
/// This is a convenience function that calls all of the `register_*_functions` functions.
/// Features must be enabled for the corresponding functions to be registered.
///
/// # Example
///
/// ```
/// # use sqlite_compressions::rusqlite::{Connection, Result};
/// # use sqlite_compressions::register_compression_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_compression_functions(&db)?;
/// # if cfg!(feature = "gzip") {
/// let result: String = db.query_row("SELECT hex(gzip('hello'))", [], |r| r.get(0))?;
/// assert_eq!(&result, "1F8B08000000000000FFCB48CDC9C9070086A6103605000000");
/// let result: String = db.query_row("SELECT hex(gzip_decode(gzip(x'0123')))", [], |r| r.get(0))?;
/// assert_eq!(&result, "0123");
/// let result: bool = db.query_row("SELECT gzip_test(gzip(x'0123'))", [], |r| r.get(0))?;
/// assert_eq!(result, true);
/// # }
/// # if cfg!(feature = "brotli") {
/// let result: String = db.query_row("SELECT hex(brotli('hello'))", [], |r| r.get(0))?;
/// assert_eq!(&result, "0B028068656C6C6F03");
/// # }
/// # Ok(())
/// # }
/// ```
pub fn register_compression_functions(conn: &Connection) -> Result<()> {
    #[cfg(feature = "gzip")]
    register_gzip_functions(conn)?;
    #[cfg(feature = "brotli")]
    register_brotli_functions(conn)?;

    Ok(())
}
