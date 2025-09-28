mod err;
mod helpers;
mod json_value_update;
mod partial_clone;
mod track_change;

pub(crate) use self::{err::*, helpers::*};
use proc_macro::TokenStream;

#[proc_macro_derive(TrackChange, attributes(track_value, track_ref, tracker))]
pub fn derive_track_change(item: TokenStream) -> TokenStream {
    extract_stream(track_change::derive_track_change_impl(item.into()))
}

#[proc_macro_derive(
    JsonValueUpdate,
    attributes(no_update, value_change, json_value_update)
)]
pub fn derive_json_value_update(item: TokenStream) -> TokenStream {
    extract_stream(json_value_update::derive_json_value_update_impl(
        item.into(),
    ))
}

#[proc_macro_derive(PartialClone, attributes(partial_clone, no_clone))]
pub fn derive_partial_clone(item: TokenStream) -> TokenStream {
    extract_stream(partial_clone::derive_partial_clone_impl(item.into()))
}

fn extract_stream(s: Result<proc_macro2::TokenStream>) -> TokenStream {
    let res = match s {
        Ok(r) => r,
        Err(e) => e.into(),
    };
    res.into()
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
