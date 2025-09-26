mod err;
mod track_change;

pub(crate) use err::*;
use proc_macro::TokenStream;

#[proc_macro_derive(TrackChange, attributes(track_value, track_ref, tracker))]
pub fn derive_track_change(item: TokenStream) -> TokenStream {
    let res = match track_change::derive_track_change_impl(item.into()) {
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
