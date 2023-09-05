/// Adds some features to existing data types
pub mod extensions;
pub mod messenger;
pub mod msg;

pub mod err;
mod save_struct_macro;

pub use err::*;
pub use save_struct_macro::*;
