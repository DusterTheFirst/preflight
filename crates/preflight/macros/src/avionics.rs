use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse::Parse, parse::ParseStream, punctuated::Punctuated, spanned::Spanned, Error, Expr,
    ExprAssign, ExprLit, ExprPath, ExprStruct, ItemImpl, Lit, LitStr, Path, Result, Token,
};

use crate::util::reconstruct;

pub fn harness(params: AvionicsParameters, input: ItemImpl) -> Result<TokenStream> {
    let (invert, trait_, _) = &input
        .trait_
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

    let st = &input.self_ty;

    let platform_impl = if let Some("testing") = option_env!("__PREFLIGHT") {
        // Running under preflight

        quote! {
            #[no_mangle]
            pub fn avionics_guide(sensors: Sensors) -> Control {
                unsafe { AVIONICS.guide(sensors) }
            }

            #[no_mangle]
            pub static __PREFLIGHT: bool = true;
        }
    } else {
        // Building

        quote! {
            #[no_mangle]
            extern "C" fn avionics_guide(sensors: Sensors) -> Control {
                unsafe { AVIONICS.guide(sensors) }
            }
        }
    };

    let default_value: TokenStream = params.default.value().parse()?;
    let default = quote_spanned! {params.default.span()=>
        #default_value
    };

    Ok(quote! {
        #input

        static mut AVIONICS: #st = #default();

        #platform_impl
    })
}

pub struct AvionicsParameters {
    default: LitStr,
}

impl Parse for AvionicsParameters {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.parse_terminated::<_, Token![,]>(ExprAssign::parse)?;

        let parsed_attributes = attributes
            .iter()
            .map(|expr| {
                if !expr.attrs.is_empty() {
                    Err(Error::new(
                        expr.span(),
                        "expressions cannot have attributes",
                    ))
                } else {
                    if let Expr::Path(ExprPath { path, attrs, .. }) = expr.left.as_ref() {
                        if !attrs.is_empty() {
                            Err(Error::new(
                                expr.span(),
                                "expressions cannot have attributes",
                            ))
                        } else {
                            if let Expr::Lit(ExprLit { lit, .. }) = expr.right.as_ref() {
                                Ok((reconstruct(&path.segments), lit))
                            } else {
                                Err(Error::new(
                                    expr.span(),
                                    "parameters can only take literals as values",
                                ))
                            }
                        }
                    } else {
                        Err(Error::new(
                            expr.span(),
                            "left side of the expression must be a path (ex. joe, or jeff:joe)",
                        ))
                    }
                }
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(AvionicsParameters {
            default: parsed_attributes
                .get("default")
                .ok_or_else(|| Error::new(attributes.span(), "missing required property `default`"))
                .and_then(|lit| {
                    if let Lit::Str(s) = lit {
                        Ok(s.clone())
                    } else {
                        Err(Error::new(
                            lit.span(),
                            "parameter `default` expects a string literal",
                        ))
                    }
                })?,
        })
    }
}
