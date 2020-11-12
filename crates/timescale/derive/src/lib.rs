use csv::StringRecord;
use parse::timescale_data_table;
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use std::{
    collections::hash_map::RandomState, collections::HashSet, fs, iter::FromIterator, path::PathBuf,
};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    AttrStyle, Expr, ExprAssign, ExprLit, ExprPath, Fields, ItemStruct, Lit, LitStr, Path,
    PathArguments, Token, Type, TypePath,
};

mod derive;
mod parse;

#[proc_macro_derive(TimescaleDataTable, attributes(csv))]
pub fn derive_timescale_data_table(input: TokenStream) -> TokenStream {
    // Parse the underlying struct
    let input: ItemStruct = parse_macro_input!(input as ItemStruct);

    derive::timescale_data_table::derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(TimescaleData, attributes(rename))]
pub fn timescale_data_loader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    TokenStream::from(match &input.fields {
        Fields::Named(fields) => {
            let path = PathBuf::from(format!(
                "{}/{}.rs.fields",
                env!("PROC_ARTIFACT_DIR"),
                input.ident.to_string()
            ));

            // FIXME: clean up, and better messages

            if let Err(e) = fs::write(
                &path,
                match fields
                    .named
                    .iter()
                    .map(|f| match &f.ty {
                        Type::Path(TypePath {
                            path: Path { segments, .. },
                            ..
                        }) => {
                            let ident = f.ident.as_ref().unwrap();
                            let ty = segments.last().unwrap();

                            let rename = f
                                .attrs
                                .iter()
                                .filter(|attr| {
                                    if let [segment] =
                                        attr.path.segments.iter().collect::<Vec<_>>().as_slice()
                                    {
                                        attr.style == AttrStyle::Outer
                                            && segment.arguments == PathArguments::None
                                            && segment.ident.to_string() == "rename"
                                    } else {
                                        false
                                    }
                                })
                                .try_fold(None::<LitStr>, |pre, a| {
                                    let input: LitStr = a.parse_args()?;

                                    if let Some(pre) = pre {
                                        Err(syn::Error::new(
                                            pre.span(),
                                            format!(
                                                "duplicate rename of field `{}`",
                                                ident.to_string()
                                            ),
                                        ))
                                    } else {
                                        Ok(Some(input))
                                    }

                                    // if let Some(st) = pre.st {
                                    //     Err(syn::Error::new(
                                    //         st.span(),
                                    //         "duplicate definition of field `st`",
                                    //     ))
                                    // } else if let Some(lit) = pre.file {
                                    //     Err(syn::Error::new(
                                    //         lit.span(),
                                    //         "duplicate definition of field `file`",
                                    //     ))
                                    // } else {
                                    //     Ok(LoadCsvArguments {
                                    //         file: input.file.or(pre.file),
                                    //         st: input.st.or(pre.st),
                                    //     })
                                    // }
                                })?;

                            let name = if let Some(rename) = rename {
                                rename.value()
                            } else {
                                ident.to_string()
                            };

                            Ok(format!("{},{},{}", ident, name, ty.ident.to_string()))
                        }
                        _ => unreachable!(),
                    })
                    .collect::<Result<Vec<_>, syn::Error>>()
                {
                    Ok(l) => l.join("\n"),
                    Err(e) => return TokenStream::from(e.to_compile_error()),
                },
            ) {
                let path = path.to_string_lossy();
                let e = e.to_string();
                quote_spanned! {input.span()=>
                    compile_error!(concat!("Failed to write to the file `", #path, "`: ", #e));
                }
            } else {
                quote! {}
            }
        }
        _ => quote_spanned! {input.span()=>
            compile_error!("struct must have named fields");
        },
    })
}

#[proc_macro_derive(ToTimescale)]
pub fn timescale_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    TokenStream::from(match input.fields {
        Fields::Named(fields) => {
            let (name, vis, generics) = (input.ident, input.vis, input.generics);
            let fields = fields.named.into_iter();

            let serializers = fields.clone().map(|f| {
                let name = f.ident;

                quote! {
                    s.serialize_field(concat!(stringify!(#name), "_x"), &self.data.#name[0])?;
                    s.serialize_field(concat!(stringify!(#name), "_y"), &self.data.#name[1])?;
                    s.serialize_field(concat!(stringify!(#name), "_z"), &self.data.#name[2])?;
                }
            });

            let fields_count = serializers.len() + 1;

            let timescale_ident = format_ident!("Timescale{}", name);

            quote! {
                #[doc(hidden)]
                #[derive(Debug)]
                #vis struct #timescale_ident #generics {
                    /// The time since the start of the simulation that this data point was logged
                    pub time: f64,
                    /// The data for this time point
                    pub data: #name #generics,
                }

                #[doc(hidden)]
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const _: () = {
                    extern crate serde as _serde;
                    extern crate timescale as _timescale;
                    use _serde::ser::SerializeStruct;
                    use _timescale::ToTimescale;

                    #[automatically_derived]
                    impl _serde::Serialize for #timescale_ident {
                        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: _serde::Serializer,
                        {
                            let mut s = serializer.serialize_struct(stringify!(#name), #fields_count)?;
                            s.serialize_field("time", &self.time)?;
                            #(#serializers)*
                            s.end()
                        }
                    }

                    #[automatically_derived]
                    impl ToTimescale for #name {
                        type Timescale = #timescale_ident;

                        fn with_time(self, time: f64) -> Self::Timescale {
                            #timescale_ident {
                                time,
                                data: self
                            }
                        }
                    }
                };
            }
        }
        _ => quote_spanned! {input.span()=>
            compile_error!("struct must have named fields");
        },
    })
}
