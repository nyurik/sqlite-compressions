use std::panic::{RefUnwindSafe, UnwindSafe};

#[cfg(feature = "trace")]
use log::trace;
use rusqlite::functions::{Context, FunctionFlags};
use rusqlite::types::{Type, ValueRef};
use rusqlite::Connection;
use rusqlite::Error::InvalidFunctionParameterType;

#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

use crate::rusqlite::Result;

pub trait Differ {
    fn diff_name() -> &'static str;
    fn patch_name() -> &'static str;
    fn diff(source: &[u8], target: &[u8]) -> Result<Vec<u8>>;
    fn patch(source: &[u8], patch: &[u8]) -> Result<Vec<u8>>;
}

pub(crate) fn register_differ<T: Differ + UnwindSafe + RefUnwindSafe + 'static>(
    conn: &Connection,
) -> Result<()> {
    // FunctionFlags derive Copy trait only in v0.31+, but we support v0.30+
    macro_rules! flags {
        () => {
            FunctionFlags::SQLITE_UTF8
                | FunctionFlags::SQLITE_DETERMINISTIC
                | FunctionFlags::SQLITE_DIRECTONLY
        };
    }

    trace!("Registering function {}", T::diff_name());
    conn.create_scalar_function(T::diff_name(), 2, flags!(), diff_fn::<T>)?;

    trace!("Registering function {}", T::patch_name());
    conn.create_scalar_function(T::patch_name(), 2, flags!(), patch_fn::<T>)
}

fn diff_fn<T: Differ + UnwindSafe + RefUnwindSafe + 'static>(
    ctx: &Context,
) -> Result<Option<Vec<u8>>> {
    let Some(source) = get_bytes(ctx, 0)? else {
        return Ok(None);
    };
    let Some(target) = get_bytes(ctx, 1)? else {
        return Ok(None);
    };
    Ok(Some(T::diff(source, target)?))
}

fn patch_fn<T: Differ + UnwindSafe + RefUnwindSafe + 'static>(
    ctx: &Context,
) -> Result<Option<Vec<u8>>> {
    let Some(source) = get_bytes(ctx, 0)? else {
        return Ok(None);
    };
    let Some(patch) = get_bytes(ctx, 1)? else {
        return Ok(None);
    };
    Ok(Some(T::patch(source, patch)?))
}

pub(crate) fn get_bytes<'a>(ctx: &'a Context, index: usize) -> Result<Option<&'a [u8]>> {
    match ctx.get_raw(index) {
        ValueRef::Blob(val) | ValueRef::Text(val) => Ok(Some(val)),
        ValueRef::Null => Ok(None),
        ValueRef::Integer(_) => Err(InvalidFunctionParameterType(index, Type::Integer)),
        ValueRef::Real(_) => Err(InvalidFunctionParameterType(index, Type::Real)),
    }
}
