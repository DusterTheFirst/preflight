use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics,
};

#[proc_macro_derive(Fields)]
pub fn fields_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    TokenStream::from(match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => {
                let name = input.ident;

                let generics = add_trait_bounds(input.generics);
                let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

                let field_names = fields.named.into_iter().map(|f| f.ident).collect::<Vec<_>>();

                quote! {
                    #[automatically_derived]
                    impl #impl_generics Fields for #name #ty_generics #where_clause {
                        fn field_names(&self) -> Option<&'static [&'static str]> {
                            Some(&[
                                #(stringify!(#field_names)),*
                            ])
                        }

                        fn field_values(&self) -> Option<&'static [std::sync::Arc<dyn std::any::Any>]> {
                            Some(&[
                                #(std::sync::Arc::new(self.#field_names.clone())),*
                            ])
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
