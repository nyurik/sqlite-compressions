use std::io::{Read, Write};

use brotli::{CompressorWriter, Decompressor};
use rusqlite::Error::UserFunctionError;

use crate::common::{register_compression, Encoder};
use crate::rusqlite::{Connection, Result};

/// Register the `brotli` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [Brotli compression](https://en.wikipedia.org/wiki/Brotli) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_compressions::rusqlite::{Connection, Result};
/// # use sqlite_compressions::register_brotli_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_brotli_functions(&db)?;
/// let result: Vec<u8> = db.query_row("SELECT brotli('hello')", [], |r| r.get(0))?;
/// let expected = b"\x0b\x02\x80\x68\x65\x6c\x6c\x6f\x03";
/// assert_eq!(result, expected);
/// let result: String = db.query_row("SELECT CAST(brotli_decode(brotli('world')) AS TEXT)", [], |r| r.get(0))?;
/// let expected = "world";
/// assert_eq!(result, expected);
/// let result: bool = db.query_row("SELECT brotli_test(brotli('world'))", [], |r| r.get(0))?;
/// let expected = true;
/// assert_eq!(result, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_brotli_functions(conn: &Connection) -> Result<()> {
    register_compression::<BrotliEncoder>(conn)
}

pub struct BrotliEncoder;

impl Encoder for BrotliEncoder {
    fn enc_name() -> &'static str {
        "brotli"
    }
    fn dec_name() -> &'static str {
        "brotli_decode"
    }
    fn test_name() -> &'static str {
        "brotli_test"
    }

    fn encode(data: &[u8], quality: Option<u32>) -> Result<Vec<u8>> {
        let mut encoder = CompressorWriter::new(Vec::new(), 4 * 1024, quality.unwrap_or(11), 22);
        encoder
            .write_all(data)
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(encoder.into_inner())
    }

    fn decode(data: &[u8]) -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        Decompressor::new(data, 4 * 1024)
            .read_to_end(&mut decompressed)
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(decompressed)
    }

    fn test(data: &[u8]) -> bool {
        // reuse the same buffer when decompressing
        // ideally we should use some null buffer, but flate2 doesn't seem to support that
        // note that buffer size does affect performance and depend on the input data size
        let mut buffer = [0u8; 4 * 1024];
        let mut decoder = Decompressor::new(data, 4 * 1024);
        while let Ok(len) = decoder.read(&mut buffer) {
            if len == 0 {
                return true;
            }
        }
        false
    }
}
