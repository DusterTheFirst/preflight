use derive::{interpolated_data, interpolated_data_table, timescale_derive};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod derive;
mod parse;
mod util;

#[proc_macro_derive(InterpolatedDataTable, attributes(table))]
pub fn derive_interpolated_data_table(input: TokenStream) -> TokenStream {
    // Parse the underlying struct
    let input: ItemStruct = parse_macro_input!(input as ItemStruct);

    interpolated_data_table::derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(InterpolatedData, attributes(data))]
pub fn interpolated_data_loader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    interpolated_data::derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(ToTimescale)]
pub fn timescale_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    timescale_derive::derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
