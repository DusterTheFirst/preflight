use crate::parse::timescale_data_table::StructArgs;
use csv::Reader;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::path::PathBuf;
use syn::{spanned::Spanned, Error, Fields, Ident, ItemStruct, LitFloat, LitStr, Path};

use super::timescale_data::{DerivedTimescaleData, TIMESCALE_DATA};

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
        return Err(Error::new(
            file.span(),
            format!("File `{}` does not exist", csv_path.to_string_lossy()),
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

    // Load in the metadata from the struct's csv file
    let (time_column_name, fields_type, fields, wanted_headers) = {
        // Read in the fields from the shared data
        let timescale_data = (*TIMESCALE_DATA).read().unwrap();
        let DerivedTimescaleData { fields, rename, ty } = timescale_data
            .get(&struct_name.to_string())
            .ok_or(Error::new(
                st.span(),
                format!("struct `{}` does not derive TimescaleData", struct_name),
            ))?
            .clone();

        (
            rename,
            Ident::new(&ty, Span::call_site()),
            fields
                .iter()
                .map(|x| Ident::new(&x.name, Span::call_site()))
                .collect::<Vec<_>>(),
            fields
                .iter()
                .map(|x| x.rename.as_ref().unwrap_or(&x.name).clone())
                .collect::<Vec<_>>(),
        )
    };

    let map_csv_error = |e: csv::Error| -> Error {
        Error::new(
            file.span(),
            format!("failed to csv file `{}`: {}", csv_path.to_string_lossy(), e),
        )
    };

    // Create reader for the csv file
    let mut csv_reader = Reader::from_path(&csv_path).map_err(map_csv_error)?;
    let mut csv_headers = csv_reader.headers().map_err(map_csv_error)?.into_iter(); // TODO: NO UNWRAPS

    // Ensure the time header/column is present
    if let Some(time_header) = time_column_name.as_ref() {
        if let Some(true) = csv_headers.next().map(|h| h == time_header) {
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
        if let Some("Time (s)") = csv_headers.next() {
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

    // Collect the rest of the headers
    let csv_headers = csv_headers.collect::<Vec<_>>();

    // Ensure all wanted fields/columns are present in the csv file
    for (i, wanted_header) in wanted_headers.iter().enumerate() {
        if let Some(csv_header) = csv_headers.get(i) {
            if csv_header != wanted_header {
                return Err(Error::new(
                    file.span(),
                    format!(
                        "header `{}` at position {} does not match expected `{}`",
                        csv_header,
                        i + 1,
                        wanted_header
                    ),
                ));
            }
        } else {
            return Err(Error::new(
                file.span(),
                format!(
                    "header `{}` (#{}) is missing from the csv file",
                    wanted_header,
                    i + 1
                ),
            ));
        }
    }

    // Error out about excess headers
    let excess_headers = &csv_headers[wanted_headers.len()..];
    if !excess_headers.is_empty() {
        return Err(Error::new(
            file.span(),
            format!(
                "csv file contained headers not defined in struct `{}`: {}",
                struct_name,
                excess_headers.join(", ")
            ),
        ));
    }

    // Read in the csv file
    let records = csv_reader
        .records()
        .map(|record| {
            record.map(|record| {
                // Read the record as a lit float
                let mut record = record
                    .iter()
                    .map(|x| LitFloat::new(&format!("{}{}", x, fields_type), Span::call_site()));

                (
                    // The first record is the time
                    record.next().unwrap(),
                    // The next records get zipped with the fields, as idents
                    record.zip(&fields).collect::<Vec<_>>(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_csv_error)?;

    fn map_fields_to_assignment((value, field): &(LitFloat, &Ident)) -> TokenStream {
        quote! {
            #field: #value
        }
    }

    let low_saturation = records.first().map(|(time, fields)| {
        let struct_fields = fields.iter().map(map_fields_to_assignment);

        quote! {
            _ if time <= #time => TimescaleData::Saturation(#struct_name {
                #(#struct_fields),*
            }),
        }
    });
    let high_saturation = records.last().map(|(time, fields)| {
        let struct_fields = fields.iter().map(map_fields_to_assignment);

        quote! {
            _ if time >= #time => TimescaleData::Saturation(#struct_name {
                #(#struct_fields),*
            }),
        }
    });

    let data_table_struct = input.ident;

    Ok(quote! {
        const _: () = {
            use timescale::TimescaleData;

            #[automatically_derived]
            impl TimescaleDataTable for #data_table_struct {
                type Datapoint = #st;
                type Time = #fields_type;

                fn get(time: Self::Time) -> TimescaleData<Self> {
                    match time {
                        #low_saturation
                        // #lerps
                        #high_saturation
                    }
                }
            }
        };
    })
}
