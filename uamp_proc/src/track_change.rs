use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Ident, Path, Token, Type, parse2, punctuated::Punctuated,
    spanned::Spanned,
};

use crate::{
    Error, Result, extract_attribute_list, extract_only_ident,
    for_each_named_field, path_to_string,
};

pub fn derive_track_change_impl(item: TokenStream) -> Result<TokenStream> {
    let item: DeriveInput = parse2(item)?;

    let mut funs = TokenStream::new();
    let mut tracker_set = false;

    for_each_named_field(item.data, |ident, field| {
        for attr in field.attrs {
            let id = path_to_string(attr.path());

            if id == "track_value" {
                funs.extend(value_field(
                    ident.clone(),
                    field.ty.clone(),
                    extract_attribute_list(attr)?,
                )?);
            } else if id == "track_ref" {
                funs.extend(ref_field(
                    ident.clone(),
                    field.ty.clone(),
                    extract_attribute_list(attr)?,
                )?);
            } else if id == "tracker" {
                if tracker_set {
                    return Error::span(
                        ident.span(),
                        "There already is tracker field.",
                    )
                    .err();
                }
                funs.extend(tracker(
                    ident.clone(),
                    extract_attribute_list(attr)?,
                )?);
                tracker_set = true;
            }
        }

        Ok(())
    })?;

    if !tracker_set {
        return Error::msg("Missing tracker field.").err();
    }

    let (impl_gen, ty_gen, wh) = item.generics.split_for_impl();
    let name = item.ident;

    Ok(quote! {
        impl #impl_gen #name #ty_gen #wh {
            #funs
        }
    })
}

fn value_field(
    id: Ident,
    ty: Type,
    attr: Punctuated<TokenStream, Token![,]>,
) -> Result<TokenStream> {
    let mut eqc = false;
    let (get_vis, set_vis) = parse_visibility(attr, |i| {
        let i: Ident = parse2(i)?;
        if i == "eq" {
            if eqc {
                Error::span(i.span(), "`eq` can be specified only once.").err()
            } else {
                eqc = true;
                Ok(true)
            }
        } else {
            Ok(false)
        }
    })?;

    let seti = format_ident!("set_{id}");
    let geti = format_ident!("get_{id}");

    let mut set = if eqc {
        quote! {
            #set_vis fn #seti(&mut self, v: #ty) {
                if v == self.#id {
                    return;
                }
                self.#id = v;
                self.set_change(true);
            }
        }
    } else {
        quote! {
            #set_vis fn #seti(&mut self, v: #ty) {
                self.#id = v;
                self.set_change(true);
            }
        }
    };

    let get = quote! {
        #get_vis fn #geti(&self) -> #ty {
            self.#id.clone()
        }
    };

    set.extend(get);

    Ok(set)
}

fn ref_field(
    id: Ident,
    ty: Type,
    attr: Punctuated<TokenStream, Token![,]>,
) -> Result<TokenStream> {
    let (get_vis, set_vis) = parse_visibility(attr, |_| Ok(false))?;

    let seti = format_ident!("mut_{id}");
    let geti = format_ident!("get_{id}");

    let res = quote! {
        #get_vis fn #geti(&self) -> &#ty {
            &self.#id
        }

        #set_vis fn #seti(&mut self) -> &mut #ty {
            &mut self.#id
        }
    };

    Ok(res)
}

fn tracker(
    id: Ident,
    attr: Punctuated<TokenStream, Token![,]>,
) -> Result<TokenStream> {
    let mut set: Option<Path> = None;
    let mut set_vis = None;
    let span = attr.span();

    for tks in attr {
        let span = tks.span();
        let Some(id) = extract_only_ident(tks.clone()) else {
            set = Some(parse2(tks)?);
            continue;
        };
        if id == "pub" || id == "priv" {
            if set_vis.is_some() {
                return Error::span(span, "Too many modifiers.").err();
            }
            set_vis = Some(id == "pub");
        } else if set.is_some() {
            return Error::span(span, "Unknown argument.").err();
        } else {
            set = Some(parse2(tks)?);
        }
    }

    let Some(set) = set else {
        return Error::span(span, "Missing setter function.").err();
    };

    let res = quote! {
        #set_vis fn set_change(&self, v: bool) {
            #set(&self.#id, v);
        }
    };

    Ok(res)
}

fn parse_visibility(
    attr: Punctuated<TokenStream, Token![,]>,
    mut other: impl FnMut(TokenStream) -> Result<bool>,
) -> Result<(TokenStream, TokenStream)> {
    let mut get_vis = None;
    let mut set_vis = None;

    for tks in attr {
        let span = tks.span();
        let Some(id) = extract_only_ident(tks.clone()) else {
            if !other(tks)? {
                return Error::span(span, "Unknown argument.").err();
            }
            continue;
        };
        let a = id.to_string();
        if a == "pub" || a == "priv" {
            let v = a == "pub";
            if get_vis.is_some() {
                if set_vis.is_some() {
                    return Error::span(span, "Too many modifiers.").err();
                }
                set_vis = Some(v);
            } else {
                get_vis = Some(v);
            }
        } else if !other(tks)? {
            return Error::span(span, "Unknown argument.").err();
        }
    }

    let get_vis = if get_vis.unwrap_or_default() {
        quote! { pub }
    } else {
        TokenStream::new()
    };
    let set_vis = if set_vis.unwrap_or_default() {
        quote! { pub }
    } else {
        TokenStream::new()
    };

    Ok((get_vis, set_vis))
}
