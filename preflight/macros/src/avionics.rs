use std::collections::HashMap;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input::ParseMacroInput,
    spanned::Spanned,
    AttributeArgs, Error, ItemImpl, Lit, Meta, MetaNameValue, NestedMeta, Result,
};

use crate::util::reconstruct;

pub fn harness(params: AvionicsParameters, input: ItemImpl) -> Result<TokenStream> {
    let (implementation, st) = {
        let ItemImpl {
            self_ty, trait_, ..
        } = &input;

        let (invert, trait_, _) = &trait_
            .as_ref()
            .ok_or_else(|| Error::new(input.span(), "no trait was found to implement"))?;

        let trait_str = {
            let mut trait_str = if trait_.leading_colon.is_some() {
                "::".to_string()
            } else {
                String::new()
            };

            trait_str.push_str(&reconstruct(&trait_.segments));

            trait_str
        };

        if !trait_str.ends_with("Avionics") {
            return Err(Error::new(
                trait_.span(),
                "expected a trait implementation of `Avionics`",
            ));
        }

        if let Some(invert) = invert {
            return Err(Error::new(
                invert.span(),
                "cannot negate the `Avionics` implementation",
            ));
        }

        (&input, self_ty)
    };

    let platform_impl = if let Some("testing") = option_env!("__PREFLIGHT") {
        // Running under preflight

        quote! {
            #[no_mangle]
            pub fn avionics_guide(sensors: &Sensors) -> Option<Control> {
                unsafe { AVIONICS.guide(sensors) }
            }

            #[no_mangle]
            pub static __PREFLIGHT: bool = true;
        }
    } else {
        // Building

        quote! {
            #[no_mangle]
            extern "C" fn avionics_guide(sensors: &Sensors) -> Option<Control> {
                unsafe { AVIONICS.guide(sensors) }
            }
        }
    };

    let default = {
        let default: TokenStream = params.default.parse()?;

        quote_spanned! {params.default.span()=>
            #default
        }
    };

    Ok(quote! {
        #implementation

        static mut AVIONICS: #st = #default();

        #platform_impl
    })
}

#[derive(Debug, FromMeta)]
pub struct AvionicsParameters {
    default: String,
    panic_handler: bool,
}

impl Parse for AvionicsParameters {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = AttributeArgs::parse(input)?;

        let mut parsed = attributes
            .into_iter()
            .map(|meta| match meta {
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    let MetaNameValue { lit, path, .. } = nv;

                    Ok((reconstruct(&path.segments), (path, lit)))
                }
                NestedMeta::Meta(m) => Err(Error::new(
                    m.span(),
                    "expected a name value pair like `a = \"b\"",
                )),
                NestedMeta::Lit(l) => Err(Error::new(
                    l.span(),
                    "expected a name value pair like `a = \"b\"",
                )),
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let params = AvionicsParameters {
            default: parsed
                .remove("default")
                .ok_or_else(|| Error::new(input.span(), "missing required property `default`"))
                .and_then(|(_, value)| {
                    if let Lit::Str(s) = value {
                        Ok(s.value())
                    } else {
                        Err(Error::new(
                            value.span(),
                            "parameter `default` expects a string",
                        ))
                    }
                })?,
            panic_handler: parsed
                .remove("panic_handler")
                .map(|(_, value)| {
                    if let Lit::Bool(b) = value {
                        Ok(b.value)
                    } else {
                        Err(Error::new(
                            value.span(),
                            "parameter `panic_handler` expects a boolean",
                        ))
                    }
                })
                .unwrap_or(Ok(true))?,
        };

        if !parsed.is_empty() {
            let unexpected = parsed.keys();
            Err(Error::new(
                input.span(),
                format!(
                    "unexpected parameter{}: {}",
                    if unexpected.len() == 1 { "" } else { "s" },
                    unexpected.cloned().collect::<Vec<String>>().join(", ")
                ),
            ))
        } else {
            Ok(params)
        }
    }
}
