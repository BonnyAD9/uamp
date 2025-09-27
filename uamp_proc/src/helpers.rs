use std::mem;

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{
    Attribute, Data, Field, Fields, Ident, Meta, Path, Token,
    parse::ParseStream, punctuated::Punctuated, spanned::Spanned,
};

use crate::{Error, Result};

pub fn for_each_named_field(
    data: Data,
    mut f: impl FnMut(Ident, Field) -> Result<()>,
) -> Result<()> {
    let Data::Struct(data) = data else {
        return Error::msg("Expected struct.").err();
    };

    let Fields::Named(fields) = data.fields else {
        return Error::span(data.fields.span(), "Expected named fields.")
            .err();
    };

    for mut field in fields.named {
        let Some(ident) = mem::take(&mut field.ident) else {
            return Error::span(field.span(), "Field must be named.").err();
        };

        f(ident, field)?;
    }

    Ok(())
}

pub fn extract_attribute_list(
    attribute: Attribute,
) -> Result<Punctuated<TokenStream, Token![,]>> {
    match attribute.meta {
        Meta::List(l) => Ok(l.parse_args_with(punctuated_streams)?),
        Meta::Path(_) => Ok(Punctuated::default()),
        _ => Error::span(attribute.span(), "Invalid attribute style.").err(),
    }
}

pub fn punctuated_streams(
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

pub fn extract_only_ident(tks: TokenStream) -> Option<Ident> {
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

pub fn get_one<I: Spanned>(
    span: Span,
    tks: impl IntoIterator<Item = I>,
) -> Result<I> {
    let mut tks = tks.into_iter();
    let one = tks.next().ok_or(Error::span(span, "One component."))?;
    if let Some(t) = tks.next() {
        Error::span(t.span(), "Expected only one argument.").err()
    } else {
        Ok(one)
    }
}

pub fn path_to_string(path: &Path) -> String {
    let mut res = String::new();
    for (i, s) in path.segments.iter().enumerate() {
        if i != 0 {
            res += "::";
        }
        res += &s.ident.to_string()
    }
    res
}
