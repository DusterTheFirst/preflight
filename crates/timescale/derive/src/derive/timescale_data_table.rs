use std::{fs, path::PathBuf};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, Fields, ItemStruct, LitStr, Path};
use crate::parse::timescale_data_table::StructArgs;

pub fn derive(input: ItemStruct) -> syn::Result<TokenStream> {
    // Ensure the struct has no fields
    if Fields::Unit != input.fields {
        return Err(Error::new(input.span(), "The struct must be a unit struct"));
    }

    // Parse the attributes
    let args = StructArgs::parse_attributes(input.attrs.as_ref())?;

    // Ensure that both args have values
    match args {
        StructArgs { file: None, .. } => Err(Error::new(
            input.ident.span(), 
            "the attribute `#[csv(path = ...)]` must be provided with a path to load csv from"
        )),
        StructArgs { st: None, .. } => Err(Error::new(
            input.ident.span(),
            "the attribute `#[csv(struct = ...)]` must be provided with the struct to deserialize the csv as"
        )),
        StructArgs {
            file: Some(file),
            st: Some(st),
        } => Ok(load_csv(file, st, input)?),
    }
}

fn load_csv(file: LitStr, st: Path, input: ItemStruct) -> syn::Result<TokenStream> {
    // Get the full path to the csv file
    let csv_path = PathBuf::from(format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        file.value()
    ));

    // Enforce the existence of the file
    if !csv_path.exists() {
        let csv_path = csv_path.to_string_lossy();
        return Err(Error::new(
            file.span(),
            format!("File `{}` does not exist", csv_path),
        ));
    }

    // Get the name of the struct to create a map of
    let struct_name = &st.segments.last().ok_or(Error::new(st.segments.span(), "path must have at least one segment"))?.ident;

    // Get the file with the fields generated from the `#[derive(TimescaleData)]` macro
    let struct_fields_path = PathBuf::from(format!(
        "{}/{}.csv",
        env!("PROC_ARTIFACT_DIR"),
        struct_name
    ));

    // Ensure the file exists
    if !struct_fields_path.exists() {
        return Err(Error::new(
            st.span(),
            format!("struct `{}` does not derive TimescaleData", struct_name),
        ));
    }

    // let reader = csv::Reader:: // TODO: CSV PARSE

    let struct_fields = fs::read_to_string(struct_fields_path).unwrap();
    // let (struct_fields, struct_fields_headers, field_type): (Vec<_>, Vec<_>, Vec<_>) =
    //     struct_fields
    //         .split("\n")
    //         .map(|r| match r.split(",").collect::<Vec<_>>()[..] {
    //             [first, second, third, ..] => (first, second, third),
    //             _ => unreachable!(),
    //         })
    //         .unzip();
    // let field_type = field_type.first().unwrap();

    // FIXME: extract into other, remove unwraps
    let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
    let mut headers = csv_reader.headers().unwrap().into_iter();

    // if let Some("Time (s)") = headers.next() {
    // } else {
    //     return TokenStream::from(quote_spanned! {file.span()=>
    //         compile_error!("csv file's first column must be exactly `Time (s)`");
    //     });
    // }

    // let headers = HashSet::<&str, RandomState>::from_iter(headers.collect::<Vec<_>>());
    // let struct_fields = HashSet::<&str, RandomState>::from_iter(struct_fields.into_iter());

    // let diff = struct_fields.difference(&headers).collect::<Vec<_>>();

    // if !diff.is_empty() {
    //     return TokenStream::from(quote_spanned! {file.span()=>
    //         compile_error!(concat!("csv columns ", #(stringify!(#diff)),*, " are missing"));
    //     });
    // }

    // let mut record = StringRecord::new();
    // while csv_reader.read_record(&mut record).unwrap() {
    //     dbg!(&record);
    // }

    // dbg!(struct_fields);

    let data_table_struct = input.ident;

    Ok(quote! {
        const _: () = {
            use timescale::TimescaleData;

            #[automatically_derived]
            impl TimescaleDataTable for #data_table_struct {
                type Datapoint = #st;

                fn get(time: f64) -> TimescaleData<Self::Datapoint> {
                    match time {

                    }
                }
            }
        };
    })
}
