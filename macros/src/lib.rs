//! .

use abs_path_core::{AbsPath, NodeName};
use proc_macro::TokenStream;
use syn::{LitStr, parse_macro_input};

/// TODO: docs.
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    match <&NodeName>::try_from(&*input.value()) {
        Ok(_) => quote::quote! {
            unsafe { ::abs_path::NodeName::from_str_unchecked(#input) }
        },
        Err(err) => syn::Error::new_spanned(input, err).into_compile_error(),
    }
    .into()
}

/// TODO: docs.
#[proc_macro]
pub fn path(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    match <&AbsPath>::try_from(&*input.value()) {
        Ok(_) => quote::quote! {
            unsafe { ::abs_path::AbsPath::from_str_unchecked(#input) }
        },
        Err(err) => syn::Error::new_spanned(input, err).into_compile_error(),
    }
    .into()
}
