use crate::parse::timescale_data_table::StructArgs;
use csv::{Reader, ReaderBuilder};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::PathBuf;
use syn::{spanned::Spanned, Error, Fields, ItemStruct, LitStr, Path};

use super::timescale_data::DerivedTimescaleData;

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
            "the attribute `#[table(path = ...)]` must be provided with a path to load csv from"
        )),
        StructArgs { st: None, .. } => Err(Error::new(
            input.ident.span(),
            "the attribute `#[table(struct = ...)]` must be provided with the struct to deserialize the csv as"
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
    let struct_name = &st
        .segments
        .last()
        .ok_or(Error::new(
            st.segments.span(),
            "path must have at least one segment",
        ))?
        .ident;

    // Get the file with the fields generated from the `#[derive(TimescaleData)]` macro
    let struct_fields_path =
        PathBuf::from(format!("{}/{}.csv", env!("PROC_ARTIFACT_DIR"), struct_name));

    // Ensure the file exists
    if !struct_fields_path.exists() {
        return Err(Error::new(
            st.span(),
            format!("struct `{}` does not derive TimescaleData", struct_name),
        ));
    }

    // Load in the metadata from the struct's csv file
    let (self_rename, fields, headers) = {
        let map_csv_error = |e: csv::Error| -> Error {
            Error::new(
                st.span(),
                format!("failed to read file `{}` produced by `#[derive(DerivedTimescaleData)]` on struct {}: {}", struct_fields_path.to_string_lossy(), struct_name,  e),
            )
        };

        // Read in all the fields from the file
        let fields = ReaderBuilder::new()
            .from_path(&struct_fields_path)
            .map_err(map_csv_error)?
            .deserialize::<DerivedTimescaleData>()
            .collect::<csv::Result<Vec<_>>>()
            .map_err(map_csv_error)?;

        // Get the first element and take it to be the rename of the time column
        if let [first, rest @ ..] = &fields[..] {
            let (fields, headers): (Vec<DerivedTimescaleData>, Vec<_>) = rest
                .iter()
                .map(|x| (x.clone(), x.rename.as_ref().cloned()))
                .unzip();

            (first.rename.as_ref().cloned(), fields, headers)
        } else {
            return Err(Error::new(file.span(), format!("failed to read file `{}` produced by `#[derive(DerivedTimescaleData)]` on struct {}", struct_fields_path.to_string_lossy(), struct_name)));
        }
    };

    let map_csv_error = |e: csv::Error| -> Error {
        Error::new(
            file.span(),
            format!("failed to csv file `{}`: {}", csv_path.to_string_lossy(), e),
        )
    };

    // Create reader for the csv file
    let mut csv_reader = Reader::from_path(&csv_path).map_err(map_csv_error)?;
    let mut headers = csv_reader.headers().map_err(map_csv_error)?.into_iter(); // TODO: NO UNWRAPS

    // Ensure the time header/column is present
    if let Some(time_header) = self_rename {
        if let Some(true) = headers.next().map(|h| h == time_header) {
        } else {
            return Err(Error::new(
                file.span(),
                format!(
                    "first column must be exactly `{}` as specified by the rename on `{}`",
                    time_header, struct_name
                ),
            ));
        }
    } else {
        if let Some("Time (s)") = headers.next() {
        } else {
            return Err(Error::new(
                file.span(),
                format!(
                    "first column must be exactly `Time (s)` unless overridden by a rename on `{}`",
                    struct_name
                ),
            ));
        }
    }

    let headers = headers.collect::<Vec<_>>();

    // Ensure all other fields/columns are present in the csv file
    for (i, field) in fields.iter().enumerate() {
        let field_name = field.rename.as_ref().unwrap_or(&field.field);
        if field_name != headers[i] {
            return Err(Error::new(
                file.span(),
                format!("header `{}` is missing from the csv file", field_name),
            ));
        }
    }

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
