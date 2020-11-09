use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{AttributeArgs, Data, DeriveInput, Fields, GenericParam, Generics, Index, ItemStruct, parse_macro_input, parse_quote, spanned::Spanned};

#[proc_macro_attribute]
pub fn timescale_data(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let args = parse_macro_input!(args as AttributeArgs);

    if let Some(arg) = args.first() {
        let len = Index::from(args.len());
        return TokenStream::from(quote_spanned! {arg.span()=>
            compile_error!(concat!("this function takes 0 arguments but ", stringify!(#len), " arguments were supplied"));
        })
    }
    
    TokenStream::from(match input.fields {
        Fields::Named(fields) => {
            let (name, attrs, vis, generics) = (input.ident, input.attrs, input.vis, input.generics);
            let fields = fields.named.into_iter();

            let serializers = fields.clone().map(|f| {
                let name = f.ident;

                quote! {
                    s.serialize_field(concat!(stringify!(#name), "_x"), &self.#name[0])?;
                    s.serialize_field(concat!(stringify!(#name), "_y"), &self.#name[1])?;
                    s.serialize_field(concat!(stringify!(#name), "_z"), &self.#name[2])?;
                }
            });

            let fields_count = serializers.len() + 1;

            quote! {
                #(#attrs)*
                #vis struct #name #generics {
                    /// The time since the start of the simulation that this data point was logged
                    pub time: f64,
                    #(#fields),*
                }

                #[doc(hidden)]
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const _: () = {
                    extern crate serde as _serde;
                    use _serde::ser::SerializeStruct;

                    impl _serde::Serialize for #name {
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
                };
            }
        }
        _ => quote_spanned! {input.span()=>
            compile_error!("Structs must have named fields");
        }
    })
}

#[proc_macro_derive(SerializeFlatten)]
pub fn serialize_flatten_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    TokenStream::from(match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => {
                let name = input.ident;

                let generics = add_trait_bounds(input.generics);
                let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

                let field_serialize = fields
                    .named
                    .iter()
                    .filter_map(|f| f.ident.as_ref())
                    .map(|field| {
                        quote! {
                            if let Some(fields) = fields::Fields::field_names(&self.#field) {
                                let values = fields::Fields::field_values(&self.#field).unwrap();
                                for field in 0..fields.len() {

                                    let value: &dyn serde::Serialize = values[field].downcast_ref().as_ref().unwrap();
                                    
                                    row.serialize_field(stringify!(fields[field]), )?;
                                }
                            } else {
                                row.serialize_field(stringify!(#field), &self.#field)?;
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                let field_count = fields.named.len();

                quote! {
                    impl #impl_generics Serialize for #name #ty_generics #where_clause {
                        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: serde::Serializer,
                        {
                            let mut row = serializer.serialize_struct(stringify!(#name), #field_count)?;

                            #(#field_serialize);*

                            // row.serialize_field("time", &self.time)?;

                            // row.serialize_field("position_x", &self.position[0])?;
                            // row.serialize_field("position_y", &self.position[1])?;
                            // row.serialize_field("position_z", &self.position[2])?;
                            row.end()
                        }
                    }
                }
            }
            _ => quote_spanned! {s.struct_token.span()=>
                compile_error!("Structs must have named fields")
            },
        },
        _ => quote_spanned! {input.span()=>
            compile_error!("Only structs are allowed for this derive macro")
        },
    })
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(fields::Fields));
        }
    }
    generics
}
