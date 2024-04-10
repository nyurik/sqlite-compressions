use std::io::{Read, Write};

use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use bzip2::Compression;
use rusqlite::Error::UserFunctionError;

use crate::common::{register_compression, Encoder};
use crate::rusqlite::{Connection, Result};

/// Register the `bzip2` SQL functions with the given `SQLite` connection.
/// The function takes a single argument and returns the [bzip2 compression](https://en.wikipedia.org/wiki/Bzip2) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_compressions::rusqlite::{Connection, Result};
/// # use sqlite_compressions::register_bzip2_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_bzip2_functions(&db)?;
/// let result: Vec<u8> = db.query_row("SELECT bzip2('hello')", [], |r| r.get(0))?;
/// let expected = b"\x42\x5a\x68\x36\x31\x41\x59\x26\x53\x59\x19\x31\x65\x3d\x00\x00\x00\x81\x00\x02\x44\xa0\x00\x21\x9a\x68\x33\x4d\x07\x33\x8b\xb9\x22\x9c\x28\x48\x0c\x98\xb2\x9e\x80";
/// assert_eq!(result, expected);
/// let result: String = db.query_row("SELECT CAST(bzip2_decode(bzip2('world')) AS TEXT)", [], |r| r.get(0))?;
/// let expected = "world";
/// assert_eq!(result, expected);
/// let result: bool = db.query_row("SELECT bzip2_test(bzip2('world'))", [], |r| r.get(0))?;
/// let expected = true;
/// assert_eq!(result, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_bzip2_functions(conn: &Connection) -> Result<()> {
    register_compression::<Bzip2Encoder>(conn)
}

pub struct Bzip2Encoder;

impl Encoder for Bzip2Encoder {
    fn enc_name() -> &'static str {
        "bzip2"
    }
    fn dec_name() -> &'static str {
        "bzip2_decode"
    }
    fn test_name() -> &'static str {
        "bzip2_test"
    }

    fn encode(data: &[u8], quality: Option<u32>) -> Result<Vec<u8>> {
        let quality = if let Some(param) = quality {
            if param > 9 {
                return Err(UserFunctionError(
                    "The optional second argument to bzip2() must be between 0 and 9".into(),
                ));
            }
            Compression::new(param)
        } else {
            Compression::default()
        };

        let mut encoder = BzEncoder::new(Vec::new(), quality);
        encoder
            .write_all(data)
            .map_err(|e| UserFunctionError(e.into()))?;
        encoder.finish().map_err(|e| UserFunctionError(e.into()))
    }

    fn decode(data: &[u8]) -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        BzDecoder::new(data)
            .read_to_end(&mut decompressed)
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(decompressed)
    }

    fn test(data: &[u8]) -> bool {
        // reuse the same buffer when decompressing
        // ideally we should use some null buffer, but bzip2 doesn't seem to support that
        // note that buffer size does affect performance and depend on the input data size
        let mut buffer = [0u8; 1024];
        let mut decoder = BzDecoder::new(data);
        while let Ok(len) = decoder.read(&mut buffer) {
            if len == 0 {
                return true;
            }
        }
        false
    }
}
