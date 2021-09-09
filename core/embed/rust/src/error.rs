use core::convert::Infallible;
use cstr_core::CStr;

use crate::micropython::{ffi, obj::Obj, qstr::Qstr};

#[derive(Debug)]
pub enum Error {
    TypeError,
    OutOfRange,
    MissingKwargs,
    CaughtException(Obj),
    KeyError(Obj),
    AttributeError(Qstr),
    ValueError(&'static CStr),
}

impl From<Error> for Obj {
    fn from(err: Error) -> Self {
        unsafe {
            match err {
                Error::TypeError => ffi::mp_obj_new_exception(&ffi::mp_type_TypeError),
                Error::OutOfRange => ffi::mp_obj_new_exception(&ffi::mp_type_OverflowError),
                Error::MissingKwargs => ffi::mp_obj_new_exception(&ffi::mp_type_TypeError),
                Error::CaughtException(obj) => obj,
                Error::KeyError(key) => {
                    ffi::mp_obj_new_exception_arg1(&ffi::mp_type_KeyError, key.into())
                }
                Error::ValueError(msg) => {
                    ffi::mp_obj_new_exception_msg(&ffi::mp_type_ValueError, msg.as_ptr())
                }
                Error::AttributeError(attr) => {
                    ffi::mp_obj_new_exception_arg1(&ffi::mp_type_AttributeError, attr.into())
                }
            }
        }
    }
}

// Implements a conversion from `core::convert::Infallible` to `Error` to so
// that code generic over `TryFrom` can work with values covered by the blanket
// impl for `Into`: `https://doc.rust-lang.org/std/convert/enum.Infallible.html`
impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
