use std::{ffi::c_void, ops::Deref, ptr};

#[repr(C)]
#[derive(Debug)]
pub struct OpaqueType {
    data: *mut c_void,
    free: unsafe extern "C" fn(*mut c_void),
}

impl OpaqueType {
    pub unsafe fn new(
        data: *mut c_void,
        free: unsafe extern "C" fn(*mut c_void),
    ) -> Self {
        Self { data, free }
    }
}

unsafe impl Send for OpaqueType {}

impl Deref for OpaqueType {
    type Target = *mut c_void;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Drop for OpaqueType {
    fn drop(&mut self) {
        if self.data.is_null() {
            unsafe { (self.free)(self.data) };
            self.data = ptr::null_mut();
        }
    }
}
