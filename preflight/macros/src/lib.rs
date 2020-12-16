use avionics::AvionicsParameters;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl};

mod avionics;
mod util;

#[proc_macro_attribute]
pub fn avionics_harness(params: TokenStream, input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(params as AvionicsParameters);
    let input = parse_macro_input!(input as ItemImpl);

    avionics::harness(params, input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
