use core::slice;
use std::{ffi::c_char, fmt::Display};

#[repr(C)]
#[derive(Debug)]
pub struct CString {
    data: *const c_char,
    len: usize,
    free: unsafe extern "C" fn(*const c_char, usize),
}

impl Display for CString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = String::from_utf8_lossy(unsafe {
            slice::from_raw_parts(self.data as *const _, self.len)
        });
        write!(f, "{str}")
    }
}

impl Drop for CString {
    fn drop(&mut self) {
        unsafe { (self.free)(self.data, self.len) };
    }
}
