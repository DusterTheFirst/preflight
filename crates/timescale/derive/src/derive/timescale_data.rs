use crate::parse::timescale_data::RenameArgs;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use syn::{Error, Fields, ItemStruct, Path, Type, TypePath, spanned::Spanned};

lazy_static! {
    pub static ref TIMESCALE_DATA: Arc<RwLock<HashMap<String, Vec<DerivedTimescaleData>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Clone)]
pub struct DerivedTimescaleData {
    pub field: String,
    pub rename: Option<String>,
    pub ty: String,
}

pub fn derive(input: ItemStruct) -> syn::Result<TokenStream> {
    match &input.fields {
        Fields::Named(fields) => {
            // Get access to the shared struct descriptor table
            let mut timescale_data = (*TIMESCALE_DATA).write().unwrap();
            let timescale_data = timescale_data.entry(input.ident.to_string()).or_default();

            // Fill in the self field
            {
                let args = RenameArgs::parse_attributes(input.attrs.as_slice(), &input.ident)?;

                timescale_data.push(DerivedTimescaleData {
                    field: "self".to_owned(),
                    rename: args.map(|args| args.rename.value()),
                    ty: input.ident.to_string(),
                });
            }

            // Fill in all other fields
            for field in &fields.named {
                if let Type::Path(TypePath {
                    path: Path { segments, .. },
                    ..
                }) = &field.ty
                {
                    // Get the information about the field
                    let ident = field.ident.as_ref().unwrap();
                    let ty = segments.last().unwrap();

                    // Parse the optional rename attribute
                    let args = RenameArgs::parse_attributes(
                        field.attrs.as_slice(),
                        field
                            .ident
                            .as_ref()
                            .ok_or(Error::new(field.span(), "field has no identifier"))?,
                    )?;

                    // Add it to the list
                    timescale_data.push(DerivedTimescaleData {
                        field: ident.to_string(),
                        rename: args.map(|args| args.rename.value()),
                        ty: ty.ident.to_string(),
                    });
                }
            }

            Ok(TokenStream::new())
        }
        _ => Err(Error::new(input.span(), "struct must have named fields")),
    }
}
