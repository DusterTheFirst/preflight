use csv::Reader;
use darling::FromDeriveInput;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::path::PathBuf;
use syn::{spanned::Spanned, Error, Ident, LitFloat, LitStr, Path};

use crate::store::{InterpolatedDataLayoutColumn, INTERPOLATED_DATA_STORE};

/// Arguments to the `derive(InterpolatedDataTable)` macro
#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_unit), attributes(table))]
pub struct InterpolatedDataTableArgs {
    pub ident: Ident,
    pub st: Path,
    pub file: LitStr,
}

pub fn derive(
    InterpolatedDataTableArgs { file, ident, st }: InterpolatedDataTableArgs,
) -> syn::Result<TokenStream> {
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
            format!("{:?} does not exist", csv_path),
        ));
    }

    // Get the name of the struct to create a map of
    let struct_name = &st
        .segments
        .last()
        .ok_or_else(|| Error::new(st.segments.span(), "path must have at least one segment"))?
        .ident;

    // Load in the metadata from the struct's csv file
    let (time_column_name, data_type, fields, wanted_headers) = {
        // Read in the fields from the shared data
        let timescale_data = (*INTERPOLATED_DATA_STORE).read().unwrap();
        let data_layout = timescale_data
            .get(&struct_name.to_string())
            .ok_or_else(|| {
                Error::new(
                    st.span(),
                    format!("struct `{}` does not derive TimescaleData", struct_name),
                )
            })?;

        (
            data_layout.time_column_name().to_owned(),
            data_layout.data_type(),
            data_layout
                .columns
                .iter()
                .map(InterpolatedDataLayoutColumn::field)
                .collect::<Vec<_>>(),
            data_layout
                .columns
                .iter()
                .map(InterpolatedDataLayoutColumn::column_name)
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        )
    };

    let map_csv_error = |e: csv::Error| -> Error {
        Error::new(
            file.span(),
            format!("failed to csv file {:?}: {}", csv_path, e),
        )
    };

    // Create reader for the csv file
    let mut csv_reader = Reader::from_path(&csv_path).map_err(map_csv_error)?;
    let mut csv_headers = csv_reader.headers().map_err(map_csv_error)?.into_iter();

    // Ensure the time header/column is present
    if let Some(h) = csv_headers.next() {
        if h != time_column_name {
            return Err(Error::new(
                file.span(),
                format!(
                    "expected first column `{}` but found column `{}`",
                    time_column_name, h
                ),
            ));
        }
    } else {
        return Err(Error::new(
            file.span(),
            format!(
                "expected first column `{}` but found nothing",
                time_column_name,
            ),
        ));
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
                    .map(|x| LitFloat::new(&format!("{}{}", x, data_type), Span::call_site()));

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
    let (low_saturation, low_value) = records
        .first()
        .map(|(time, fields)| {
            let struct_fields = fields.iter().map(map_fields_to_assignment);

            (
                quote! {
                    _ if time <= #time => InterpolatedDataPoint::Saturation(#struct_name {
                        #(#struct_fields),*
                    }),
                },
                time,
            )
        })
        .ok_or_else(|| Error::new(file.span(), "csv file must have at least one entry of data"))?;

    // Generate the high saturation from the last datapoint
    let (high_saturation, high_value) = records
        .last()
        .map(|(time, fields)| {
            let struct_fields = fields.iter().map(map_fields_to_assignment);

            (
                quote! {
                    _ if time >= #time => InterpolatedDataPoint::Saturation(#struct_name {
                        #(#struct_fields),*
                    }),
                },
                time,
            )
        })
        .ok_or_else(|| Error::new(file.span(), "csv file must have at least one entry of data"))?;

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

    let csv_path = csv_path.to_string_lossy();

    Ok(quote! {
        const _: () = {
            use timescale::InterpolatedDataPoint;

            include_bytes!(#csv_path);

            #[automatically_derived]
            impl InterpolatedDataTable for #ident {
                type Datapoint = #st;
                type Time = #data_type;

                const MIN: Self::Time = #low_value;
                const MAX: Self::Time = #high_value;

                fn get_raw(time: Self::Time) -> InterpolatedDataPoint<Self::Datapoint, Self::Time> {
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
