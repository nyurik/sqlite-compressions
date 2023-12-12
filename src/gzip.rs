use std::io::{Read, Write};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::Error::UserFunctionError;

use crate::common::{register_compression, Encoder};
use crate::rusqlite::{Connection, Result};

/// Register the `gzip` SQL functions with the given `SQLite` connection.
/// The function takes a single argument and returns the [GZIP compression](https://en.wikipedia.org/wiki/Gzip) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_compressions::rusqlite::{Connection, Result};
/// # use sqlite_compressions::register_gzip_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_gzip_functions(&db)?;
/// let result: Vec<u8> = db.query_row("SELECT gzip('hello')", [], |r| r.get(0))?;
/// let expected = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\xff\xcb\x48\xcd\xc9\xc9\x07\x00\x86\xa6\x10\x36\x05\x00\x00\x00";
/// assert_eq!(result, expected);
/// let result: String = db.query_row("SELECT CAST(gzip_decode(gzip('world')) AS TEXT)", [], |r| r.get(0))?;
/// let expected = "world";
/// assert_eq!(result, expected);
/// let result: bool = db.query_row("SELECT gzip_test(gzip('world'))", [], |r| r.get(0))?;
/// let expected = true;
/// assert_eq!(result, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_gzip_functions(conn: &Connection) -> Result<()> {
    register_compression::<GzipEncoder>(conn)
}

struct GzipEncoder;

impl Encoder for GzipEncoder {
    fn enc_name() -> &'static str {
        "gzip"
    }
    fn dec_name() -> &'static str {
        "gzip_decode"
    }
    fn test_name() -> &'static str {
        "gzip_test"
    }

    fn encode(data: &[u8], quality: Option<u32>) -> Result<Vec<u8>> {
        let quality = if let Some(param) = quality {
            if param > 9 {
                return Err(UserFunctionError(
                    "The optional second argument to gzip() must be between 0 and 9".into(),
                ));
            }
            Compression::new(param)
        } else {
            Compression::default()
        };

        let mut encoder = GzEncoder::new(Vec::new(), quality);
        encoder
            .write_all(data)
            .map_err(|e| UserFunctionError(e.into()))?;
        encoder.finish().map_err(|e| UserFunctionError(e.into()))
    }

    fn decode(data: &[u8]) -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        GzDecoder::new(data)
            .read_to_end(&mut decompressed)
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(decompressed)
    }
}
