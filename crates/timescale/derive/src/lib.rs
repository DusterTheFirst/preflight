use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, ExprPath, Fields, Index, ItemStruct};

#[proc_macro]
pub fn load_csv(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExprPath);

    TokenStream::from(quote! {})
}

#[proc_macro_derive(Lerp)]
pub fn lerp_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let name = &input.ident;

    TokenStream::from(match input.fields {
        Fields::Named(fields) => {
            let fields = fields.named.into_iter().map(|f| {
                let name = f.ident;

                quote! {
                    #name: self.#name.lerp(other.#name, t)
                }
            });

            quote! {
                #[doc(hidden)]
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const _: () = {
                    extern crate serde as _serde;
                    extern crate timescale as _timescale;
                    use _timescale::Lerp;

                    #[automatically_derived]
                    impl Lerp<f64> for #name {
                        fn lerp(self, other: Self, t: f64) -> Self {
                            Self {
                                #(#fields),*
                            }
                        }
                    }
                };
            }
        }
        Fields::Unnamed(fields) => {
            let fields = fields.unnamed.into_iter().enumerate().map(|(i, f)| {
                let name = Index::from(i);

                quote! {
                    self.#name.lerp(other.#name, t)
                }
            });

            quote! {
                #[automatically_derived]
                impl Lerp<f64> for #name {
                    fn lerp(self, other: Self, t: f64) -> Self {
                        Self (
                            #(#fields),*
                        )
                    }
                }
            }
        }
        _ => quote_spanned! {input.span()=>
            compile_error!("Structs must have fields");
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
