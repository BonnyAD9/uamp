use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Msg(Cow<'static, str>),
    #[error("{1}")]
    Span(Span, Cow<'static, str>),
    #[error(transparent)]
    Syn(#[from] syn::Error),
}

impl Error {
    pub fn msg(value: impl Into<Cow<'static, str>>) -> Self {
        Self::Msg(value.into())
    }

    pub fn span(span: Span, value: impl Into<Cow<'static, str>>) -> Self {
        Self::Span(span, value.into())
    }

    pub fn err<T>(self) -> Result<T> {
        Err(self)
    }

    pub fn get_span(&self) -> Span {
        match self {
            Self::Msg(_) => Span::call_site(),
            Self::Span(s, _) => *s,
            Self::Syn(e) => e.span(),
        }
    }
}

impl From<Error> for TokenStream {
    fn from(value: Error) -> Self {
        if let Error::Syn(e) = value {
            return e.into_compile_error();
        }
        let msg = value.to_string();
        let span = value.get_span();
        quote_spanned! { span =>
            compile_error!(#msg);
        }
    }
}
