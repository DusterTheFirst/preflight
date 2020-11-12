use crate::parse::timescale_data::RenameArgs;
use csv::WriterBuilder;
use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syn::{spanned::Spanned, Error, Fields, ItemStruct, Path, Type, TypePath};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DerivedTimescaleData {
    pub field: String,
    pub rename: Option<String>,
    #[serde(rename = "type")]
    pub ty: String,
}

pub fn derive(input: ItemStruct) -> syn::Result<TokenStream> {
    match &input.fields {
        Fields::Named(fields) => {
            // Create the path to the descriptor for this struct
            let path = PathBuf::from(format!(
                "{}/{}.csv",
                env!("PROC_ARTIFACT_DIR"),
                input.ident.to_string()
            ));

            let map_csv_error = |e: csv::Error| -> Error {
                Error::new(
                    input.ident.span(),
                    format!("failed to write file `{}`: {}", path.to_string_lossy(), e),
                )
            };

            let mut writer = WriterBuilder::new()
                .from_path(&path)
                .map_err(map_csv_error)?;

            {
                let args = RenameArgs::parse_attributes(input.attrs.as_slice(), &input.ident)?;

                writer
                    .serialize(DerivedTimescaleData {
                        field: "self".to_owned(),
                        rename: args.map(|args| args.rename.value()),
                        ty: input.ident.to_string(),
                    })
                    .map_err(map_csv_error)?;
            }

            for field in &fields.named {
                if let Type::Path(TypePath {
                    path: Path { segments, .. },
                    ..
                }) = &field.ty
                {
                    let ident = field.ident.as_ref().unwrap();
                    let ty = segments.last().unwrap();

                    let args = RenameArgs::parse_attributes(
                        field.attrs.as_slice(),
                        field
                            .ident
                            .as_ref()
                            .ok_or(Error::new(field.span(), "field has no identifier"))?,
                    )?;

                    writer
                        .serialize(DerivedTimescaleData {
                            field: ident.to_string(),
                            rename: args.map(|args| args.rename.value()),
                            ty: ty.ident.to_string(),
                        })
                        .map_err(map_csv_error)?;
                }
            }

            Ok(TokenStream::new())
        }
        _ => Err(Error::new(input.span(), "struct must have named fields")),
    }
}
