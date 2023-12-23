use std::panic::{RefUnwindSafe, UnwindSafe};

#[cfg(feature = "trace")]
pub(crate) use log::trace;
use rusqlite::functions::Context;
use rusqlite::types::{Type, ValueRef};
use rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};

use crate::rusqlite::functions::FunctionFlags;
use crate::rusqlite::{Connection, Result};
#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

pub trait Encoder {
    fn enc_name() -> &'static str;
    fn dec_name() -> &'static str;
    fn test_name() -> &'static str;
    fn encode(data: &[u8], quality: Option<u32>) -> Result<Vec<u8>>;
    fn decode(data: &[u8]) -> Result<Vec<u8>>;
    fn test(data: &[u8]) -> bool;
    // {
    //     Self::decode(data).is_ok()
    // }
}

pub(crate) fn register_compression<T: Encoder + UnwindSafe + RefUnwindSafe + 'static>(
    conn: &Connection,
) -> Result<()> {
    trace!("Registering function {}", T::enc_name());
    conn.create_scalar_function(
        T::enc_name(),
        -1,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_DIRECTONLY,
        encoder_fn::<T>,
    )?;

    trace!("Registering function {}", T::dec_name());
    conn.create_scalar_function(
        T::dec_name(),
        -1,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_DIRECTONLY,
        decoder_fn::<T>,
    )?;

    trace!("Registering function {}", T::test_name());
    conn.create_scalar_function(
        T::test_name(),
        -1,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_DIRECTONLY,
        testing_fn::<T>,
    )
}

fn encoder_fn<T: Encoder + UnwindSafe + RefUnwindSafe + 'static>(
    ctx: &Context,
) -> Result<Option<Vec<u8>>> {
    let param_count = ctx.len();
    if param_count == 0 || param_count > 2 {
        return Err(InvalidParameterCount(param_count, 1));
    }
    let quality = if param_count == 2 {
        Some(ctx.get::<u32>(1)?)
    } else {
        None
    };

    let value = ctx.get_raw(0);
    match value {
        ValueRef::Blob(val) => {
            trace!("{}: encoding blob {val:?}", T::enc_name());
            Ok(Some(T::encode(val, quality)?))
        }
        ValueRef::Text(val) => {
            trace!("{}: encoding text {val:?}", T::enc_name());
            Ok(Some(T::encode(val, quality)?))
        }
        ValueRef::Null => {
            trace!("{}: ignoring NULL", T::enc_name());
            Ok(None)
        }
        #[allow(unused_variables)]
        ValueRef::Integer(val) => {
            trace!("{}: unsupported Integer {val:?}", T::enc_name());
            Err(InvalidFunctionParameterType(0, Type::Integer))
        }
        #[allow(unused_variables)]
        ValueRef::Real(val) => {
            trace!("{}: unsupported Real {val:?}", T::enc_name());
            Err(InvalidFunctionParameterType(0, Type::Real))
        }
    }
}

fn decoder_fn<T: Encoder + UnwindSafe + RefUnwindSafe + 'static>(
    ctx: &Context,
) -> Result<Option<Vec<u8>>> {
    let param_count = ctx.len();
    if param_count != 1 {
        return Err(InvalidParameterCount(param_count, 1));
    }

    let value = ctx.get_raw(0);
    match value {
        ValueRef::Blob(val) => {
            trace!("{}: decoding blob {val:?}", T::dec_name());
            Ok(Some(T::decode(val)?))
        }
        ValueRef::Null => {
            trace!("{}: ignoring NULL", T::dec_name());
            Ok(None)
        }
        #[allow(unused_variables)]
        ValueRef::Text(val) => {
            trace!("{}: unsupported Text {val:?}", T::dec_name());
            Err(InvalidFunctionParameterType(0, Type::Text))
        }
        #[allow(unused_variables)]
        ValueRef::Integer(val) => {
            trace!("{}: unsupported Integer {val:?}", T::dec_name());
            Err(InvalidFunctionParameterType(0, Type::Integer))
        }
        #[allow(unused_variables)]
        ValueRef::Real(val) => {
            trace!("{}: unsupported Real {val:?}", T::dec_name());
            Err(InvalidFunctionParameterType(0, Type::Real))
        }
    }
}

fn testing_fn<T: Encoder + UnwindSafe + RefUnwindSafe + 'static>(
    ctx: &Context,
) -> Result<Option<bool>> {
    let param_count = ctx.len();
    if param_count != 1 {
        return Err(InvalidParameterCount(param_count, 1));
    }

    let value = ctx.get_raw(0);
    match value {
        ValueRef::Blob(val) => {
            trace!("{}: testing encoded blob {val:?}", T::test_name());
            Ok(Some(T::test(val)))
        }
        ValueRef::Null => {
            trace!("{}: ignoring NULL", T::test_name());
            Ok(None)
        }
        #[allow(unused_variables)]
        ValueRef::Text(val) => {
            trace!("{}: unsupported Text {val:?}", T::test_name());
            Err(InvalidFunctionParameterType(0, Type::Text))
        }
        #[allow(unused_variables)]
        ValueRef::Integer(val) => {
            trace!("{}: unsupported Integer {val:?}", T::test_name());
            Err(InvalidFunctionParameterType(0, Type::Integer))
        }
        #[allow(unused_variables)]
        ValueRef::Real(val) => {
            trace!("{}: unsupported Real {val:?}", T::test_name());
            Err(InvalidFunctionParameterType(0, Type::Real))
        }
    }
}
