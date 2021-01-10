use darling::{ast, util::Ignored, FromDeriveInput, FromField};
use proc_macro2::{Ident, TokenStream};
use store::{InterpolatedDataLayout, InterpolatedDataLayoutColumn, INTERPOLATED_DATA_STORE};
use syn::{spanned::Spanned, Error, LitStr, Path, Result, Type, TypePath};

use crate::store;

/// Attr level arguments to the `derive(InterpolatedData)` macro
#[derive(Debug, FromField)]
#[darling(attributes(data))]
pub struct InterpolatedDataDeriveField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default)]
    pub rename: Option<LitStr>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(data), supports(struct_named))]
pub struct InterpolatedDataDerive {
    pub ident: Ident,
    #[darling(default)]
    pub rename: Option<LitStr>,
    pub data: ast::Data<Ignored, InterpolatedDataDeriveField>,
}

pub fn derive(input: InterpolatedDataDerive) -> Result<TokenStream> {
    let fields = input.data.take_struct().unwrap().fields;

    let mut data_type = None;
    let mut columns = Vec::with_capacity(fields.len());

    // Fill in the other fields
    for InterpolatedDataDeriveField { ident, rename, ty } in fields {
        if let Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) = &ty
        {
            // Get the type of the field
            let input_type = &segments.last().unwrap().ident;

            // Ensure that the type matches the prior
            if let Some(ref data_type) = data_type {
                if input_type != data_type {
                    return Err(Error::new(ty.span(), format!("all fields in the struct must be the same type, found conflicting types {} and {}", input_type, data_type)));
                }
            } else {
                data_type.replace(input_type.clone());
            }

            // Add the column to the array
            columns.push(InterpolatedDataLayoutColumn::new(ident.unwrap(), rename));
        } else {
            return Err(Error::new(ty.span(), "invalid data type"));
        }
    }

    if let Some(data_type) = data_type {
        let mut timescale_data = INTERPOLATED_DATA_STORE.write().unwrap();
        timescale_data.insert(
            input.ident.to_string(),
            InterpolatedDataLayout::new(data_type, input.rename, columns),
        );

        Ok(TokenStream::new())
    } else {
        Err(Error::new(
            input.ident.span(),
            "struct must contain one or more fields",
        ))
    }

    // match &input.fields {
    //     Fields::Named(input_fields) => {
    //         let mut ty = None;
    //         let mut rename = None;
    //         let mut fields = Vec::new();

    //         // Fill in the self field
    //         {
    //             if let Some(args) =
    //                 InterpolatedDataDerive::parse_attributes(input.attrs.as_slice(), &input.ident)?
    //             {
    //                 rename.replace(args.rename.value());
    //             }
    //         }

    //         // Fill in all other fields
    //         for field in &input_fields.named {
    //             if let Type::Path(TypePath {
    //                 path: Path { segments, .. },
    //                 ..
    //             }) = &field.ty
    //             {
    //                 // Get the information about the field
    //                 let ident = field.ident.as_ref().unwrap(); // The field is in a named struct, it must have a name
    //                 let input_ty = segments.last().unwrap(); // There must be at least one segment in the path for it to be a valid struct

    //                 // Parse the optional rename attribute
    //                 let args = InterpolatedDataArgs::parse_attributes(
    //                     field.attrs.as_slice(),
    //                     field
    //                         .ident
    //                         .as_ref()
    //                         .ok_or_else(|| Error::new(field.span(), "field has no identifier"))?,
    //                 )?;

    //                 fields.push(TimescaleFieldData {
    //                     name: ident.to_string(),
    //                     rename: args.map(|args| args.rename.value()),
    //                 });

    //                 if let Some(ref ty) = ty {
    //                     if ty != &input_ty.ident.to_string() {
    //                         return Err(Error::new(input_ty.span(), format!("all fields in the struct must be the same type, found types {} and {}", ty, input_ty.ident)));
    //                     }
    //                 } else {
    //                     ty.replace(input_ty.ident.to_string());
    //                 }
    //             }
    //         }

    //         match ty {
    //             Some(ty) => {
    //                 // Get access to the shared struct descriptor table
    //                 let mut timescale_data = (*INTERPOLATED_DATA).write().unwrap();

    //                 timescale_data.insert(
    //                     input.ident.to_string(),
    //                     InterpolatedData { ty, fields, rename },
    //                 );

    //                 Ok(TokenStream::new())
    //             }
    //             None => Err(Error::new(
    //                 input_fields.span(),
    //                 "struct must contain one or more fields",
    //             )),
    //         }
    //     }
    //     _ => Err(Error::new(input.span(), "struct must have named fields")),
    // }
}
