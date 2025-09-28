use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse2, spanned::Spanned};

use crate::{
    Error, Result, extract_attribute_list, for_each_named_field, get_one,
    path_to_string,
};

pub fn derive_json_value_update_impl(
    item: TokenStream,
) -> Result<TokenStream> {
    let item: DeriveInput = parse2(item)?;

    let mut arms = TokenStream::new();

    for_each_named_field(item.data, |ident, field| {
        let mut val_chng = None;

        for attr in field.attrs {
            let id = path_to_string(attr.path());

            if id == "no_update" {
                return Ok(());
            } else if id == "value_change" {
                val_chng = Some(get_one(
                    attr.path().span(),
                    extract_attribute_list(attr)?,
                )?);
            }
        }

        let id_str = ident.to_string();

        let arm = if let Some(val_chng) = val_chng {
            quote! {
                #id_str => {
                    self.#ident = from_value(v)?;
                    changes.add_change(#val_chng);
                }
            }
        } else {
            quote! {
                #id_str => self.#ident = from_value(v)?,
            }
        };

        arms.extend([arm]);

        Ok(())
    })?;

    let mut changes = None;
    let mut ty = None;
    let mut err = None;

    for attr in item.attrs {
        let span = attr.path().span();
        let id = path_to_string(attr.path());

        if id != "json_value_update" {
            continue;
        }

        if ty.is_some() {
            return Error::span(span, "Expected this only once.").err();
        }

        let mut args = extract_attribute_list(attr)?.into_iter();

        let Some(arg) = args.next() else {
            return Error::span(span, "Expected result type.").err();
        };
        ty = Some(arg);

        let Some(arg) = args.next() else {
            return Error::span(span, "Expected Error.").err();
        };
        err = Some(arg);

        if let Some(arg) = args.next() {
            changes = Some(quote! {
                let mut changes = #arg;
            });
        }

        if let Some(arg) = args.next() {
            return Error::span(arg.span(), "Expected at most 2 arguments.")
                .err();
        }
    }

    let Some(ty) = ty else {
        return Error::msg("Missing attribute `json_value_update` on struct.")
            .err();
    };

    let changes = changes.unwrap_or_else(|| quote! { let changes = (); });

    let name = item.ident;
    let (impl_gen, ty_gen, wh) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_gen #name #ty_gen #wh {
            fn update_from_json_object(
                &mut self,
                obj: serde_json::Map<String, serde_json::Value>
            ) -> #ty {
                use serde_json::from_value;

                #changes

                for (k, v) in obj {
                    match k.as_str() {
                        #arms
                        #err
                    }
                }

                Ok(changes)
            }
        }
    })
}
