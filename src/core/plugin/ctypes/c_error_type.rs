#[repr(C)]
#[derive(Debug)]
#[allow(dead_code)] // It is constructed by external plugins
pub enum CErrorType {
    NoError,
    Recoverable,
    Fatal,
}
