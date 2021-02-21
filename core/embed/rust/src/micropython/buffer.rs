use core::{convert::TryFrom, ops::Deref, ptr, slice};

use crate::{error::Error, micropython::obj::Obj};

use super::ffi;

pub struct Buffer {
    ptr: *const u8,
    len: usize,
}

impl TryFrom<Obj> for Buffer {
    type Error = Error;

    fn try_from(obj: Obj) -> Result<Self, Self::Error> {
        let mut bufinfo = ffi::mp_buffer_info_t {
            buf: ptr::null_mut(),
            len: 0,
            typecode: 0,
        };
        if unsafe { ffi::mp_get_buffer(obj, &mut bufinfo, ffi::MP_BUFFER_READ as _) } {
            Ok(Self {
                ptr: bufinfo.buf as _,
                len: bufinfo.len as _,
            })
        } else {
            Err(Error::NotBuffer)
        }
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.to_slice()
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        self.to_slice()
    }
}

impl Buffer {
    fn to_slice(&self) -> &[u8] {
        if self.ptr.is_null() {
            // `ptr` can be null if len == 0.
            &[]
        } else {
            // SAFETY: We assume that `ptr` is pointing to memory:
            //  - immutable for the whole lifetime of immutable ref of `self`.
            //  - of length `len` bytes.
            unsafe { slice::from_raw_parts(self.ptr, self.len) }
        }
    }
}