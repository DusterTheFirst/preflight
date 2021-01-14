//! Macros for preflight

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use avionics::AvionicsParameters;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemImpl};

mod avionics;

/// Harness to connect hardware agnostic flight systems to firmware or to the
/// `preflight_cargo` utility
#[proc_macro_attribute]
pub fn avionics_harness(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    let output = parse_macro_input::parse::<AttributeArgs>(args)
        .map_err(|err| err.to_compile_error())
        .and_then(|args| AvionicsParameters::from_list(&args).map_err(|err| err.write_errors()))
        .and_then(|params| avionics::harness(params, &input).map_err(|err| err.to_compile_error()))
        .unwrap_or_else(|e| e);

    (quote! {
        #input

        #output
    })
    .into()
}
