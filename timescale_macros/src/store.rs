use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;
use proc_macro2::{Ident, Span};
use syn::LitStr;

lazy_static! {
    pub static ref INTERPOLATED_DATA_STORE: Arc<RwLock<HashMap<String, InterpolatedDataLayout>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

/// Layout of data to be loaded into an interpolated data table
#[derive(Debug, Clone)]
pub struct InterpolatedDataLayout {
    /// Columns of data to load
    pub columns: Vec<InterpolatedDataLayoutColumn>,
    /// Data type that the values will be parsed in as
    data_type: String,
    /// Optional rename of the time column from the default `Time (s)`
    time_column_rename: Option<String>,
}

impl InterpolatedDataLayout {
    pub fn new(
        data_type: Ident,
        time_column_rename: Option<LitStr>,
        columns: Vec<InterpolatedDataLayoutColumn>,
    ) -> Self {
        Self {
            columns,
            data_type: data_type.to_string(),
            time_column_rename: time_column_rename.map(|x| x.value()),
        }
    }

    /// data type that the values will be parsed in as
    pub fn data_type(&self) -> Ident {
        Ident::new(&self.data_type, Span::call_site())
    }

    /// Name of the time column
    pub fn time_column_name(&self) -> &str {
        self.time_column_rename
            .as_ref()
            .map(String::as_str)
            .unwrap_or("Time (s)")
    }
}

/// Column of interpolated timescale data
#[derive(Debug, Clone)]
pub struct InterpolatedDataLayoutColumn {
    /// Structure's field to populate with the data
    field: String,
    /// Optional rename of the column in the data
    column_rename: Option<String>,
}

impl InterpolatedDataLayoutColumn {
    pub fn new(field: Ident, column_rename: Option<LitStr>) -> Self {
        Self {
            field: field.to_string(),
            column_rename: column_rename.map(|x| x.value()),
        }
    }

    /// Structure's field to populate with the data
    pub fn field(&self) -> Ident {
        Ident::new(&self.field, Span::call_site())
    }

    /// Name of the column in the data
    pub fn column_name(&self) -> &str {
        self.column_rename
            .as_ref()
            .map(String::as_ref)
            .unwrap_or(&self.field)
    }
}
