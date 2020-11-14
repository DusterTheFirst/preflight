use super::interpolated_data::INTERPOLATED_DATA;
use crate::{
    derive::interpolated_data::InterpolatedData,
    parse::interpolated_data_table::InterpolatedDataTableArgs,
};
use csv::Reader;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::path::PathBuf;
use syn::{spanned::Spanned, Error, Fields, Ident, ItemStruct, LitFloat, LitStr, Path};

pub fn derive(input: ItemStruct) -> syn::Result<TokenStream> {
    // Ensure the struct has no fields
    if Fields::Unit != input.fields {
        return Err(Error::new(input.span(), "The struct must be a unit struct"));
    }

    // Parse the attributes
    let args = InterpolatedDataTableArgs::parse_attributes(input.attrs.as_ref())?;

    // Ensure that both args have values
    match args {
        InterpolatedDataTableArgs { file: None, .. } => Err(Error::new(
            input.ident.span(),
            "the attribute `#[table(path = ...)]` must be provided with a path to load csv from"
        )),
        InterpolatedDataTableArgs { st: None, .. } => Err(Error::new(
            input.ident.span(),
            "the attribute `#[table(struct = ...)]` must be provided with the struct to deserialize the csv as"
        )),
        InterpolatedDataTableArgs {
            file: Some(file),
            st: Some(st),
        } => Ok(load_csv(file, st, input)?),
    }
}

fn load_csv(file: LitStr, st: Path, input: ItemStruct) -> syn::Result<TokenStream> {
    // Get the full path to the csv file
    let csv_path = PathBuf::from(format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("environment variable `CARGO_MANIFEST_DIR` must be set"),
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
        let timescale_data = (*INTERPOLATED_DATA).read().unwrap();
        let InterpolatedData { fields, rename, ty } = timescale_data
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
    let mut csv_headers = csv_reader.headers().map_err(map_csv_error)?.into_iter();

    // Ensure the time header/column is present
    if let Some(time_header) = time_column_name.as_ref() {
        if let Some(h) = csv_headers.next() {
            if h != time_header {
                return Err(Error::new(
                    file.span(),
                    format!(
                        "expected first column `{}` as specified by the rename on `{}` but found column `{}`",
                        time_header, struct_name, h
                    ),
                ));
            }
        } else {
            return Err(Error::new(
                file.span(),
                format!(
                    "expected first column `{}` as specified by the rename on `{}` but found nothing",
                    time_header, struct_name
                ),
            ));
        }
    } else {
        if let Some(h) = csv_headers.next() {
            if "Time (s)" != h {
                return Err(Error::new(
                    file.span(),
                    format!(
                        "expected first column `Time (s)` but found `{}`\n\n\t\tnote = you can override this column name using the `#[timescale(rename = \"new name\")]` attribute on the struct `{}`",
                        h, struct_name
                    ),
                ));
            }
        } else {
            return Err(Error::new(
                file.span(),
                format!("expected first column `Time (s)` but found nothing",),
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

    /// Map a value and field into a token stream were the field is a field of a struct
    /// and the value is its value
    fn map_fields_to_assignment((value, field): &(LitFloat, &Ident)) -> TokenStream {
        quote! {
            #field: #value
        }
    }

    // Generate the low saturation from the first datapoint
    let low_saturation = records.first().map(|(time, fields)| {
        let struct_fields = fields.iter().map(map_fields_to_assignment);

        quote! {
            _ if time <= #time => InterpolatedDataPoint::Saturation(#struct_name {
                #(#struct_fields),*
            }),
        }
    });

    // Generate the high saturation from the last datapoint
    let high_saturation = records.last().map(|(time, fields)| {
        let struct_fields = fields.iter().map(map_fields_to_assignment);

        quote! {
            _ if time >= #time => InterpolatedDataPoint::Saturation(#struct_name {
                #(#struct_fields),*
            }),
        }
    });

    // Only generate the linear interpolations if there are more than one record to interpolate between
    let lerps = if records.len() == 1 {
        Vec::new()
    } else {
        let mut lerps = Vec::new();

        // Iterate over each of the records in the csv file
        for (i, (low_time, low_fields)) in records.iter().enumerate() {
            // Get the next record to lerp to, breaking the loop if we have run out of records to lerp to
            if let Some((high_time, high_fields)) = records.get(i + 1) {
                // Map the fields to a token stream
                let low_fields = low_fields.iter().map(map_fields_to_assignment);
                let high_fields = high_fields.iter().map(map_fields_to_assignment);

                // Add the lerp case to the match statement
                lerps.push(quote! {
                    _ if time >= #low_time && time < #high_time => {
                        InterpolatedDataPoint::Interpolation {
                            next: #struct_name {
                                #(#high_fields),*
                            },
                            prev: #struct_name {
                                #(#low_fields),*
                            },
                            percent: (time - #low_time) / (#high_time - #low_time),
                        }
                    }
                });
            } else {
                break;
            }
        }

        lerps
    };

    let data_table_struct = input.ident;

    Ok(quote! {
        const _: () = {
            use timescale::InterpolatedDataPoint;
            
            #[automatically_derived]
            impl InterpolatedDataTable for #data_table_struct {
                type Datapoint = #st;
                type Time = #fields_type;

                fn get_raw(time: Self::Time) -> InterpolatedDataPoint<Self> {
                    match time {
                        #low_saturation
                        #(#lerps),*
                        #high_saturation
                        _ => unreachable!(),
                    }
                }
            }
        };
    })
}
