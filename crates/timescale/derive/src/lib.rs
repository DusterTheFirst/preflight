use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    AttrStyle, Expr, ExprAssign, ExprLit, ExprPath, Fields, ItemStruct, Lit, LitStr, Path,
    PathArguments, Token,
};

#[derive(Default, Debug)]
struct LoadCsvArguments {
    st: Option<Path>,
    file: Option<LitStr>,
}

impl Parse for LoadCsvArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let arguments: Punctuated<ExprAssign, Token![,]> =
            input.parse_terminated(ExprAssign::parse)?;

        Ok(arguments
            .into_iter()
            .try_fold(LoadCsvArguments::default(), |pre, expr| {
                if let Expr::Path(ExprPath { path, .. } ) = *expr.left {
                    let name = path.segments.iter().map(|seg| {
                        seg.ident.to_string()
                    }).collect::<Vec<_>>().join("::");

                    match (name.as_str(), *expr.right) {
                        ("st", Expr::Path(ExprPath { path , .. })) => {
                            if pre.st.is_some() {
                                Err(syn::Error::new(
                                    path.span(),
                                    "duplicate definition of field `st`",
                                ))
                            } else {
                                Ok(LoadCsvArguments {
                                    st: Some(path),
                                    ..pre
                                })
                            }
                        },
                        ("file", Expr::Lit(ExprLit { lit: Lit::Str(str_lit), .. })) => {
                            if pre.file.is_some() {
                                Err(syn::Error::new(
                                    str_lit.span(),
                                    "duplicate definition of field `file`",
                                ))
                            } else {
                                Ok(LoadCsvArguments {
                                    file: Some(str_lit),
                                    ..pre
                                })
                            }
                        }
                        (name, _) => Err(syn::Error::new(
                            path.segments.span(),
                            format!("unknown argument `{}`.\navailable arguments are `file` and `st`", name),
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        expr.span(),
                        "invalid argument syntax.\nexpected syntax `argument_name = \"argument_value\"`",
                    ))
                }
            })?)
    }
}

#[proc_macro_derive(TimescaleDataTable, attributes(csv))]
pub fn load_csv(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    if Fields::Unit != input.fields {
        return TokenStream::from(quote_spanned! {input.span()=>
            compile_error!("The struct must be a unit struct");
        });
    }

    let attributes = input
        .attrs
        .iter()
        .filter(|attr| {
            if let [segment] = attr.path.segments.iter().collect::<Vec<_>>().as_slice() {
                attr.style == AttrStyle::Outer
                    && segment.arguments == PathArguments::None
                    && segment.ident.to_string() == "csv"
            } else {
                false
            }
        })
        .try_fold::<_, _, syn::Result<_>>(LoadCsvArguments::default(), |pre, a| {
            let input = a.parse_args::<LoadCsvArguments>()?;

            if let Some(st) = pre.st {
                Err(syn::Error::new(
                    st.span(),
                    "duplicate definition of field `st`",
                ))
            } else if let Some(lit) = pre.file {
                Err(syn::Error::new(
                    lit.span(),
                    "duplicate definition of field `file`",
                ))
            } else {
                Ok(LoadCsvArguments {
                    file: input.file.or(pre.file),
                    st: input.st.or(pre.st),
                })
            }
        });

    TokenStream::from(match attributes {
        Err(e) => e.to_compile_error(),
        Ok(attributes) => match attributes {
            LoadCsvArguments { file: None, .. } => {
                quote_spanned! {input.ident.span()=>
                    compile_error!("a path to load csv from using the attribute `#[csv(path = ...)]` must be provided");
                }
            }
            LoadCsvArguments { st: None, .. } => {
                quote_spanned! {input.ident.span()=>
                    compile_error!("struct to deserialize the csv as using the attribute `#[csv(struct = ...)] must be provided`");
                }
            }
            LoadCsvArguments {
                file: Some(file),
                st: Some(st),
            } => {
                let path = PathBuf::from(format!(
                    "{}/{}",
                    std::env::var("CARGO_MANIFEST_DIR")
                        .expect("env var CARGO_MANIFEST_DIR missing"),
                    file.value()
                ));

                if !path.exists() {
                    let path = path.to_string_lossy();
                    return TokenStream::from(quote_spanned! {file.span()=>
                        compile_error!(concat!("File ", #path, " does not exist"));
                    });
                }

                let data_table_struct = input.ident;

                quote! {
                    const _: () = {
                        use timescale::TimescaleData;

                        #[automatically_derived]
                        impl TimescaleDataTable for #data_table_struct {
                            type Datapoint = #st;

                            fn get(time: f64) -> TimescaleData<Self::Datapoint> {
                                match time {
                                   
                                }
                            }
                        }
                    };
                }
            }
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
            compile_error!("Structs must have named fields");
        },
    })
}
