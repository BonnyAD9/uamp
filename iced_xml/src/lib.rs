use proc_macro::{TokenStream, token_stream::IntoIter, TokenTree, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Data};

#[proc_macro]
pub fn iced_xml(input: TokenStream) -> TokenStream {
    let text = input.to_string();
    let input = input.into_iter();

    TokenStream::new()
}

fn parse_element(tokens: &mut IntoIter) -> Option<TokenStream> {
    let t = match tokens.next() {
        Some(t) => t,
        None => return None
    };

    let span = t.span();

    Some(match t {
        TokenTree::Punct(p) => {
            
        },
        _ => quote!(span=> compile_error!("expected '<'")).into(),
    })
}
