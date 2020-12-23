#![forbid(unsafe_code)]

use avionics::AvionicsParameters;
use darling::FromMeta;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemImpl};

mod avionics;
mod util;

#[proc_macro_attribute]
pub fn avionics_harness(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemImpl);

    match AvionicsParameters::from_list(&args) {
        Err(err) => err.write_errors(),
        Ok(params) => avionics::harness(params, input).unwrap_or_else(|err| err.to_compile_error()),
    }
    .into()
}
