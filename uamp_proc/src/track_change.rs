use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, Ident, Meta, Path, Token, Type,
    parse::ParseStream, parse2, punctuated::Punctuated, spanned::Spanned,
};

use crate::{Error, Result};

pub fn derive_track_change_impl(item: TokenStream) -> Result<TokenStream> {
    let item: DeriveInput = syn::parse2(item)?;
    let Data::Struct(data) = item.data else {
        return Error::msg("TrackChange is supported only for structs.").err();
    };

    let Fields::Named(fields) = data.fields else {
        return Error::msg("TrackChange supports only named fields.").err();
    };

    let mut funs = TokenStream::new();
    let mut tracker_set = false;

    for field in fields.named {
        let Some(ident) = field.ident else {
            return Error::span(field.span(), "Field must be named.").err();
        };

        for attr in field.attrs {
            let Some(id) = attr.path().get_ident() else {
                continue;
            };
            let id = id.to_string();
            let args = match attr.meta {
                Meta::List(l) => l.parse_args_with(punctuated_streams)?,
                Meta::Path(_) => Punctuated::default(),
                _ => continue,
            };
            if id == "track_value" {
                funs.extend(value_field(
                    ident.clone(),
                    field.ty.clone(),
                    args,
                )?);
            } else if id == "track_ref" {
                funs.extend(ref_field(ident.clone(), field.ty.clone(), args)?);
            } else if id == "tracker" {
                if tracker_set {
                    return Error::span(
                        ident.span(),
                        "There already is tracker field.",
                    )
                    .err();
                }
                funs.extend(tracker(ident.clone(), args)?);
                tracker_set = true;
            }
        }
    }

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

    let seti = format_ident!("set_{id}");
    let geti = format_ident!("mut_{id}");

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

fn extract_only_ident(tks: TokenStream) -> Option<Ident> {
    let mut tks = tks.into_iter();
    let id = tks.next()?;
    if tks.next().is_some() {
        return None;
    }

    match id {
        TokenTree::Ident(id) => Some(id),
        _ => None,
    }
}

fn punctuated_streams(
    input: ParseStream,
) -> syn::Result<Punctuated<TokenStream, Token![,]>> {
    let mut res = Punctuated::new();
    loop {
        let mut tokens = TokenStream::new();

        while !input.is_empty() && !input.peek(Token![,]) {
            tokens.extend([input.parse::<TokenTree>()?]);
        }

        if tokens.is_empty() && input.is_empty() {
            break;
        }

        res.push_value(tokens);

        let Ok(c) = input.parse() else {
            break;
        };
        res.push_punct(c);
    }

    Ok(res)
}
