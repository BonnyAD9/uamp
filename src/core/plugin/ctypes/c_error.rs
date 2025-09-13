use crate::core::plugin::ctypes::CString;

#[repr(C)]
pub struct CError {
    pub msg: CString,
    pub typ: i32,
}
