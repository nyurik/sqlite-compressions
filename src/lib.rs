#![cfg_attr(feature = "default", doc = include_str!("../README.md"))]
//
// Unsafe code is required for cdylib, so only use it for this crate
#![forbid(unsafe_code)]

#[cfg(not(any(
    feature = "brotli",
    feature = "bsdiff4",
    feature = "bsdiffraw",
    feature = "bzip2",
    feature = "gzip",
)))]
compile_error!(
    "At least one of these features must be enabled: gzip, brotli, bzip2, bsdiff4, bsdiffraw"
);

/// Re-export of the [`rusqlite`](https://crates.io/crates/rusqlite) crate to avoid version conflicts.
pub use rusqlite;

use crate::rusqlite::{Connection, Result};

#[cfg(any(feature = "bsdiff4", feature = "bsdiffraw"))]
mod common_diff;
#[cfg(any(feature = "bsdiff4", feature = "bsdiffraw"))]
pub use crate::common_diff::Differ;

#[cfg(any(feature = "brotli", feature = "bzip2", feature = "gzip"))]
mod common;
#[cfg(any(feature = "brotli", feature = "bzip2", feature = "gzip"))]
pub use crate::common::Encoder;

#[cfg(feature = "bsdiff4")]
mod bsdiff4;
#[cfg(feature = "bsdiff4")]
pub use crate::bsdiff4::{register_bsdiff4_functions, Bsdiff4Differ};

#[cfg(feature = "bsdiffraw")]
mod bsdiffraw;
#[cfg(feature = "bsdiffraw")]
pub use crate::bsdiffraw::{register_bsdiffraw_functions, BsdiffRawDiffer};

#[cfg(feature = "brotli")]
mod brotli;
#[cfg(feature = "brotli")]
pub use crate::brotli::{register_brotli_functions, BrotliEncoder};

#[cfg(feature = "bzip2")]
mod bzip2;
#[cfg(feature = "bzip2")]
pub use crate::bzip2::{register_bzip2_functions, Bzip2Encoder};

#[cfg(feature = "gzip")]
mod gzip;
#[cfg(feature = "gzip")]
pub use crate::gzip::{register_gzip_functions, GzipEncoder};

/// Register all compression functions for the given `SQLite` connection.
/// This is a convenience function that calls all the `register_*_functions` functions.
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
    #[cfg(feature = "bzip2")]
    register_bzip2_functions(conn)?;
    #[cfg(feature = "bsdiff4")]
    register_bsdiff4_functions(conn)?;
    #[cfg(feature = "bsdiffraw")]
    register_bsdiffraw_functions(conn)?;

    Ok(())
}
