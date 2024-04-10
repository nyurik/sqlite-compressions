use std::io::Cursor;

#[cfg(feature = "trace")]
use log::trace;
use qbsdiff::bsdiff::Bsdiff;
use qbsdiff::bspatch::Bspatch;
use rusqlite::functions::{Context, FunctionFlags};
use rusqlite::types::{Type, ValueRef};
use rusqlite::Error::{InvalidFunctionParameterType, UserFunctionError};

use crate::rusqlite::{Connection, Result};

#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

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
    let flags = FunctionFlags::SQLITE_UTF8
        | FunctionFlags::SQLITE_DETERMINISTIC
        | FunctionFlags::SQLITE_DIRECTONLY;

    trace!("Registering function bsdiff4");
    conn.create_scalar_function("bsdiff4", 2, flags, |ctx| {
        let Some(source) = get_bytes(ctx, 0)? else {
            return Ok(None);
        };
        let Some(target) = get_bytes(ctx, 1)? else {
            return Ok(None);
        };
        let mut patch = Vec::new();
        Bsdiff::new(source, target)
            .compare(Cursor::new(&mut patch))
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(Some(patch))
    })?;

    trace!("Registering function bspatch4");
    conn.create_scalar_function("bspatch4", 2, flags, |ctx| {
        let Some(source) = get_bytes(ctx, 0)? else {
            return Ok(None);
        };
        let Some(patch) = get_bytes(ctx, 1)? else {
            return Ok(None);
        };
        let mut target = Vec::new();
        Bspatch::new(patch)
            .and_then(|patch| patch.apply(source, Cursor::new(&mut target)))
            .map_err(|e| UserFunctionError(e.into()))?;
        Ok(Some(target))
    })
}

fn get_bytes<'a>(ctx: &'a Context, index: usize) -> Result<Option<&'a [u8]>> {
    match ctx.get_raw(index) {
        ValueRef::Blob(val) | ValueRef::Text(val) => Ok(Some(val)),
        ValueRef::Null => Ok(None),
        ValueRef::Integer(_) => Err(InvalidFunctionParameterType(index, Type::Integer)),
        ValueRef::Real(_) => Err(InvalidFunctionParameterType(index, Type::Real)),
    }
}
