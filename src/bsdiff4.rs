use std::io::Cursor;

use qbsdiff::bsdiff::Bsdiff;
use qbsdiff::bspatch::Bspatch;
use rusqlite::Error::UserFunctionError;

use crate::common_diff::{register_differ, Differ};
use crate::rusqlite::{Connection, Result};

/// Register the `bsdiff4` and `bspatch4` SQL functions with the given `SQLite` connection.
/// The `bsdiff4` function takes two arguments, and returns the [BSDiff delta](https://github.com/mendsley/bsdiff#readme) (blob) of the binary difference.
/// The arguments can be either a string or a blob.
/// If any of the arguments are `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_compressions::rusqlite::{Connection, Result};
/// # use sqlite_compressions::register_bsdiff4_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_bsdiff4_functions(&db)?;
/// let result: Vec<u8> = db.query_row("SELECT bspatch4('013479', bsdiff4('013479', '23456789'))", [], |r| r.get(0))?;
/// let expected = b"23456789";
/// assert_eq!(result, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_bsdiff4_functions(conn: &Connection) -> Result<()> {
    register_differ::<Bsdiff4Differ>(conn)
}

pub struct Bsdiff4Differ;

impl Differ for Bsdiff4Differ {
    fn diff_name() -> &'static str {
        "bsdiff4"
    }

    fn patch_name() -> &'static str {
        "bspatch4"
    }

    fn diff(source: &[u8], target: &[u8]) -> Result<Vec<u8>> {
        let mut patch = Vec::new();
        Bsdiff::new(source, target)
            .compare(Cursor::new(&mut patch))
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(patch)
    }

    fn patch(source: &[u8], patch: &[u8]) -> Result<Vec<u8>> {
        let mut target = Vec::new();
        Bspatch::new(patch)
            .and_then(|patch| patch.apply(source, Cursor::new(&mut target)))
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(target)
    }
}
