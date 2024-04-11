use std::io::Cursor;

use rusqlite::Error::UserFunctionError;

use crate::common_diff::{register_differ, Differ};
use crate::rusqlite::{Connection, Result};

/// Register the `bsdiffraw` and `bspatchraw` SQL functions with the given `SQLite` connection.
/// The `bsdiffraw` function takes two arguments, and returns the [BSDiff delta](https://github.com/mendsley/bsdiff#readme) (blob) of the binary difference.
/// The arguments can be either a string or a blob.
/// If any of the arguments are `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_compressions::rusqlite::{Connection, Result};
/// # use sqlite_compressions::register_bsdiffraw_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_bsdiffraw_functions(&db)?;
/// let result: String = db.query_row("SELECT hex(bsdiffraw('abc013479zz', 'abc23456789zzf'))", [], |r| r.get(0))?;
/// assert_eq!(result.as_str(), "03000000000000000B00000000000000070000000000000000000032333435363738397A7A66");
/// let result: String = db.query_row("SELECT hex(bspatchraw('abc013479zz', bsdiffraw('abc013479zz', 'abc23456789zzf')))", [], |r| r.get(0))?;
/// assert_eq!(result.as_str(), "61626332333435363738397A7A66");
/// let result: Vec<u8> = db.query_row("SELECT bspatchraw('013479', bsdiffraw('013479', '23456789'))", [], |r| r.get(0))?;
/// let expected = b"23456789";
/// assert_eq!(result, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_bsdiffraw_functions(conn: &Connection) -> Result<()> {
    register_differ::<BsdiffRawDiffer>(conn)
}

pub struct BsdiffRawDiffer;

impl Differ for BsdiffRawDiffer {
    fn diff_name() -> &'static str {
        "bsdiffraw"
    }

    fn patch_name() -> &'static str {
        "bspatchraw"
    }

    fn diff(source: &[u8], target: &[u8]) -> Result<Vec<u8>> {
        let mut patch = Vec::new();
        bsdiff::diff(source, target, &mut patch).map_err(|e| UserFunctionError(e.into()))?;
        Ok(patch)
    }

    fn patch(source: &[u8], patch: &[u8]) -> Result<Vec<u8>> {
        let mut target = Vec::new();
        bsdiff::patch(source, &mut Cursor::new(patch), &mut target)
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        let source = b"abc013479zz";
        let target = b"abc23456789zzf";
        let patch = BsdiffRawDiffer::diff(source, target).unwrap();
        let expected = b"\x03\x00\x00\x00\x00\x00\x00\x00\x0B\x00\x00\x00\x00\x00\x00\x00\x07\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x32\x33\x34\x35\x36\x37\x38\x39\x7A\x7A\x66";
        assert_eq!(patch, expected);
    }

    #[test]
    fn test_patch() {
        let source = b"abc013479zz";
        let patch = b"\x03\x00\x00\x00\x00\x00\x00\x00\x0B\x00\x00\x00\x00\x00\x00\x00\x07\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x32\x33\x34\x35\x36\x37\x38\x39\x7A\x7A\x66";
        let target = BsdiffRawDiffer::patch(source, patch).unwrap();
        let expected = b"abc23456789zzf";
        assert_eq!(target, expected);
    }
}
