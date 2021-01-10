//! Macros for use with the timescale crate
//!
//! This crate is re-exported by timescale and should almost never be used alone

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use darling::FromDeriveInput;
use derive::{interpolated_data, interpolated_data_table, timescale_derive};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

mod derive;
mod store;

/// Load a csv file into a static data table with support for linearly interpolating
/// data points between the discreet time intervals
///
/// This derive macro goes hand and hand with the [`InterpolatedData`] derive
/// macro. Data loaded into this table needs to be decorated with the aforementioned
/// [`InterpolatedData`] derive macro so that their structure can be available to
/// this macro at build time.
///
/// # Arguments
/// This macro requires 2 arguments.
/// - `st` describes the structure (that derives [`InterpolatedData`]) to represent
/// the timescale data.
/// - `file` describes the csv file to read in the timescale data from
///
/// # Example
/// See the data in this [csv file] for context for the following example
///
/// ```
/// # use lerp::Lerp;
/// use timescale::{InterpolatedData, InterpolatedDataTable};
///
/// #[derive(Debug, Lerp, InterpolatedData)]
/// pub struct RocketEngine {
///     #[data(rename = "Thrust (N)")]
///     pub thrust: f64,
/// }
///
/// /// The thrust curve of an Estes A8 rocket motor
/// #[derive(InterpolatedDataTable)]
/// #[table(file = "../assets/motors/Estes_A8.csv", st = "RocketEngine")]
/// pub struct EstesA8;
///
/// fn main() {
///     assert_eq!(EstesA8::get(0.35).thrust, 3.813); // Exact value from data
///     assert_eq!(EstesA8::get(0.4).thrust, 3.9468823529411763); // Linear interpolated estimate
///     assert_eq!(EstesA8::get(1.0).thrust, 0.0); // Saturated at 0
/// }
/// ```
///
/// [csv file]: https://github.com/DusterTheFirst/preflight/blob/master/assets/motors/Estes_A8.csv
#[proc_macro_derive(InterpolatedDataTable, attributes(table))]
pub fn derive_interpolated_data_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match FromDeriveInput::from_derive_input(&input) {
        Err(err) => err.write_errors(),
        Ok(args) => {
            interpolated_data_table::derive(args).unwrap_or_else(|err| err.to_compile_error())
        }
    }
    .into()
}

/// Load the layout of a structure to be consumed by [`InterpolatedDataTable`]
///
/// This structure is useless on its own and produces no useable output. This macro
/// exists to provide compile-time information about the struct it marks to the
/// [`InterpolatedDataTable`] macro
///
/// # Arguments
/// This macro allows for 1 argument to be provided on either the structure itself
/// or on the fields of the structure.
/// - `rename` allows you to set the column name as it appears in the csv file
/// different than the name of the field in the rust structure. If applied to the
/// structure itself, it will rename the time column's header from the default
/// `Time (s)`
///
/// # The Nitty Gritty
/// This macro exists because macros are not able to transverse outside of their
/// given token tree (easily). This macro will fill up a HashMap in memory with
/// the layouts of any structs marked with this macro which will be searched through
/// when the [`InterpolatedDataTable`] macro generates its table at compile time.
/// Yes, this derive macro could be an attribute macro, but keeping it as a derive
/// macro allows for consistency with the other macros and stops the macro from
/// consuming the structure provided.
#[proc_macro_derive(InterpolatedData, attributes(data))]
pub fn interpolated_data_loader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match FromDeriveInput::from_derive_input(&input) {
        Err(err) => err.write_errors(),
        Ok(args) => interpolated_data::derive(args).unwrap_or_else(|err| err.to_compile_error()),
    }
    .into()
}

/// Automatically derive the `ToTimescale` trait and generate the expanded
/// timescale structure.
///
/// # Example
/// ```no_run
/// use nalgebra::Vector3;
/// use timescale::ToTimescale;
///
/// #[derive(Debug, Clone, ToTimescale)]
/// struct VectorDatapoint {
///     position: Vector3<f64>,
///     velocity: Vector3<f64>,
///     acceleration: Vector3<f64>,
///     net_force: Vector3<f64>,
/// }
/// ```
#[proc_macro_derive(ToTimescale)]
pub fn timescale_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    timescale_derive::derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
