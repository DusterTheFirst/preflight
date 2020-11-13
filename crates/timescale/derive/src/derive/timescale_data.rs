use crate::parse::timescale_data::RenameArgs;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use syn::{spanned::Spanned, Error, Fields, ItemStruct, Path, Type, TypePath};

lazy_static! {
    pub static ref TIMESCALE_DATA: Arc<RwLock<HashMap<String, DerivedTimescaleData>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Clone)]
pub struct DerivedTimescaleData {
    pub fields: Vec<TimescaleFieldData>,
    pub ty: String,
    pub rename: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TimescaleFieldData {
    pub name: String,
    pub rename: Option<String>,
}

pub fn derive(input: ItemStruct) -> syn::Result<TokenStream> {
    match &input.fields {
        Fields::Named(input_fields) => {
            let mut ty = None;
            let mut rename = None;
            let mut fields = Vec::new();

            // Fill in the self field
            {
                if let Some(args) =
                    RenameArgs::parse_attributes(input.attrs.as_slice(), &input.ident)?
                {
                    rename.replace(args.rename.value());
                }
            }

            // Fill in all other fields
            for field in &input_fields.named {
                if let Type::Path(TypePath {
                    path: Path { segments, .. },
                    ..
                }) = &field.ty
                {
                    // Get the information about the field
                    let ident = field.ident.as_ref().unwrap();
                    let input_ty = segments.last().unwrap();

                    // Parse the optional rename attribute
                    let args = RenameArgs::parse_attributes(
                        field.attrs.as_slice(),
                        field
                            .ident
                            .as_ref()
                            .ok_or(Error::new(field.span(), "field has no identifier"))?,
                    )?;

                    fields.push(TimescaleFieldData {
                        name: ident.to_string(),
                        rename: args.map(|args| args.rename.value()),
                    });

                    if let Some(ref ty) = ty {
                        if ty != &input_ty.ident.to_string() {
                            return Err(Error::new(input_ty.span(), format!("all fields in the struct must be the same type, found types {} and {}", ty, input_ty.ident)));
                        }
                    } else {
                        ty.replace(input_ty.ident.to_string());
                    }
                }
            }

            match ty {
                Some(ty) => {
                    // Get access to the shared struct descriptor table
                    let mut timescale_data = (*TIMESCALE_DATA).write().unwrap();
                    timescale_data.insert(
                        input.ident.to_string(),
                        DerivedTimescaleData { ty, fields, rename },
                    );

                    Ok(TokenStream::new())
                }
                None => Err(Error::new(
                    input_fields.span(),
                    "struct must contain one or more fields",
                )),
            }
        }
        _ => Err(Error::new(input.span(), "struct must have named fields")),
    }
}
