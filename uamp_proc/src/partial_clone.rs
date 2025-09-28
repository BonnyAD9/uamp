use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse2, spanned::Spanned};

use crate::{
    Error, Result, extract_attribute_list, extract_only_ident,
    for_each_named_field, path_to_string,
};

pub fn derive_partial_clone_impl(item: TokenStream) -> Result<TokenStream> {
    let item: DeriveInput = parse2(item)?;

    let mut fields = TokenStream::new();
    let mut clones = TokenStream::new();

    for_each_named_field(item.data, |ident, field| {
        for attr in field.attrs {
            let id = path_to_string(attr.path());
            if id == "no_clone" {
                return Ok(());
            }
        }

        let ty = field.ty;

        fields.extend(quote! {
            pub #ident: #ty,
        });

        clones.extend(quote! {
            #ident: self.#ident.clone(),
        });

        Ok(())
    })?;

    let mut vis = None;
    let mut clone_ty = None;

    for attr in item.attrs {
        let id = path_to_string(attr.path());

        if id != "partial_clone" {
            continue;
        }

        let span = attr.span();

        if vis.is_some() {
            return Error::span(span, "Multiple attributes.").err();
        }

        let mut args = extract_attribute_list(attr)?.into_iter();

        let Some(arg) = args.next() else {
            return Error::span(span, "Expected visibility and struct name.")
                .err();
        };
        vis = Some(arg);

        let Some(arg) = args.next() else {
            return Error::span(span, "Expected visibility and struct name.")
                .err();
        };
        clone_ty = Some(arg);

        if let Some(arg) = args.next() {
            return Error::span(arg.span(), "Too many arguments.").err();
        }
    }

    let Some(vis) = vis else {
        return Error::msg("Missing attribute `partial_clone` on struct.")
            .err();
    };

    let vis_span = vis.span();
    let Some(vis) = extract_only_ident(vis) else {
        return Error::span(vis_span, "Expected either pub or priv.").err();
    };

    let vis = if vis == "pub" {
        quote! { pub }
    } else if vis == "priv" {
        TokenStream::new()
    } else {
        return Error::span(vis.span(), "Expected either pub or prov.").err();
    };

    let name = item.ident;
    let (impl_gen, ty_gen, wh) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_gen #name #ty_gen #wh {
            #vis fn partial_clone(&self) -> #clone_ty {
                #clone_ty {
                    #clones
                }
            }
        }

        #vis struct #clone_ty {
            #fields
        }
    })
}
