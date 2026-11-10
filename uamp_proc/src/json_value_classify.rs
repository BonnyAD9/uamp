use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse2, spanned::Spanned};

use crate::{
    Error, Result, extract_attribute_list, for_each_named_field, get_one,
    path_to_string,
};

pub fn derive_json_value_classify_impl(
    item: TokenStream,
) -> Result<TokenStream> {
    let item: DeriveInput = parse2(item)?;

    let mut def = quote! { Default::default() };
    let mut ty = None;
    for attr in item.attrs {
        let id = path_to_string(attr.path());

        if id == "class_default" {
            def = get_one(attr.path().span(), extract_attribute_list(attr)?)?;
        } else if id == "class_type" {
            ty = Some(get_one(
                attr.path().span(),
                extract_attribute_list(attr)?,
            )?);
        }
    }

    let Some(ty) = ty else {
        return Err(Error::msg("Missing class type."));
    };

    let mut arms = TokenStream::new();

    for_each_named_field(item.data, |ident, field| {
        let mut class = None;
        for attr in field.attrs {
            let id = path_to_string(attr.path());

            if id != "class" {
                continue;
            }
            class = Some(get_one(
                attr.path().span(),
                extract_attribute_list(attr)?,
            )?);
        }

        let id_str = ident.to_string();
        let class = class.as_ref().unwrap_or(&def);

        arms.extend(quote! {
            #id_str => #class,
        });

        Ok(())
    })?;

    arms.extend(quote! {
        _ => #def,
    });

    let name = item.ident;
    let (impl_gen, ty_gen, wh) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_gen #name #ty_gen #wh {
            fn classify(obj: &serde_json::Map<String, serde_json::Value>) -> #ty {
                obj.iter().map(|(k, _)| {
                    match k.as_str() {
                        #arms
                    }
                }).max().unwrap_or_else(|| #def)
            }
        }
    })
}
