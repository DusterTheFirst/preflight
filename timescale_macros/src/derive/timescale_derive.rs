use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Error, Fields, ItemStruct, Result};

pub fn derive(input: ItemStruct) -> Result<TokenStream> {
    match input.fields {
        Fields::Named(fields) => {
            // Get the metadata needed
            let (name, vis, generics) = (input.ident, input.vis, input.generics);

            // Create serialization for each vector field
            let serializers = fields.named.into_iter().map(|f| {
                let name = f.ident;

                quote! {
                    s.serialize_field(concat!(stringify!(#name), "_x"), &self.data.#name[0])?;
                    s.serialize_field(concat!(stringify!(#name), "_y"), &self.data.#name[1])?;
                    s.serialize_field(concat!(stringify!(#name), "_z"), &self.data.#name[2])?;
                }
            });

            // Get the fields count from the length of the serializers, plus the one for time
            let fields_count = serializers.len() * 3 + 1;

            // Create the timescale struct ident
            let timescale_ident = format_ident!("Timescale{}", name);

            Ok(quote! {
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
            })
        }
        _ => Err(Error::new(input.span(), "struct must have named fields")),
    }
}
