use std::mem;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, LitStr, Token, parse2, punctuated::Punctuated,
    spanned::Spanned,
};

use crate::{
    Error, Result, extract_attribute_list, for_each_variant, path_to_string,
};

pub fn variant_array(item: TokenStream) -> Result<TokenStream> {
    let mut enu: DeriveInput = parse2(item)?;
    let Some(attr) = mem::take(&mut enu.attrs)
        .into_iter()
        .find(|a| path_to_string(a.path()) == "variant_array")
    else {
        return Error::msg("Missing variant_array attribute").err();
    };

    let Some(name) = extract_attribute_list(attr)?.into_iter().next() else {
        return Error::msg("Missing name for variant_array.").err();
    };

    let name: syn::Ident = parse2(name)?;

    let mut items = vec![];

    for_each_variant(enu.data, |mut v| {
        let Some(attr) = mem::take(&mut v.attrs)
            .into_iter()
            .find(|a| path_to_string(a.path()) == "list_name")
        else {
            return Error::span(v.span(), "Missing name for list.").err();
        };

        for arg in extract_attribute_list(attr)? {
            let s: syn::LitStr = parse2(arg)?;
            items.push(s);
        }

        Ok(())
    })?;

    let mut item_tokens = Punctuated::<LitStr, Token![,]>::new();
    item_tokens.extend(items);

    let list = quote! {
        pub const #name: &[&str] = &[#item_tokens];
    };

    Ok(list)
}
