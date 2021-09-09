use core::mem::MaybeUninit;

use crate::error::Error;

use super::{ffi, obj::Obj};

pub fn raise_exception(err: Error) -> ! {
    unsafe {
        ffi::nlr_jump(Obj::from(err).as_ptr());
    }
    panic!();
}

/// Execute `func` while catching MicroPython exceptions. Returns `Ok` in the
/// successful case, and `Err` with the caught `Obj` in case of a raise.
pub fn except<F, T>(mut func: F) -> Result<T, Error>
where
    F: FnMut() -> T,
{
    // Because MicroPython exceptions use `setjmp` and `longjmp`-like mechanism that
    // doesn't play too well with Rust, we setup the non-local return pads in C, and
    // execute `func` through a callback.

    unsafe {
        // First, we craft a wrapping closure that calls `func`. Because we are generic
        // over the return type, we cannot pass the returned value over the FFI
        // boundary, so we assign it explicitly in `wrapper`.
        let mut result = MaybeUninit::zeroed();
        let mut wrapper = || {
            result = MaybeUninit::new(func());
        };
        // `wrapper` is a closure, and to pass it over the FFI, we split it into a
        // function pointer, and a user-data pointer.
        // `ffi::trezor_obj_call_protected` then calls the `callback` with the
        // `argument`.
        let (callback, argument) = split_func_into_callback_and_argument(&mut wrapper);
        let exception = ffi::trezor_obj_call_protected(Some(callback), argument);
        if exception == Obj::const_none() {
            Ok(result.assume_init())
        } else {
            Err(Error::CaughtException(exception))
        }
    }
}

type ProtectedArgument = *mut cty::c_void;
type ProtectedCallback = unsafe extern "C" fn(ProtectedArgument);

fn split_func_into_callback_and_argument<F>(func: &mut F) -> (ProtectedCallback, ProtectedArgument)
where
    F: FnMut(),
{
    // Here we mono-morphize a version of `trampoline` for each type `F`, so it
    // calls the correct `FnMut` impl, and cast `func` into its data part to use
    // as the argument.
    (trampoline::<F>, func as *mut _ as *mut _)
}

unsafe extern "C" fn trampoline<F>(arg: ProtectedArgument)
where
    F: FnMut(),
{
    // Synthesize a callable `*mut F` from the closure environment pointer `arg`,
    // and call it.
    let func = arg as *mut F;
    unsafe {
        (*func)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn except_returns_ok_on_no_exception() {
        let result = except(|| 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn except_catches_value_error() {
        let result = except(|| raise_exception(Error::TypeError));
        assert!(result.is_err());
    }
}
