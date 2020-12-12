use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse::Parse, parse::ParseStream, spanned::Spanned, Error, Expr, ExprAssign, ExprLit, ExprPath,
    ImplItem, ItemImpl, Lit, LitStr, Result, Token,
};

use crate::util::reconstruct;

pub fn harness(params: AvionicsParameters, input: ItemImpl) -> Result<TokenStream> {
    let (implementation, st) = {
        let ItemImpl {
            attrs,
            generics,
            items,
            self_ty,
            trait_,
            unsafety,
            ..
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

        (
            if params.no_panic {
                let items = items.into_iter().map(|item| match item {
                    ImplItem::Method(method) => {
                        quote! {
                            #[preflight_impl::no_panic]
                            #method
                        }
                    }
                    _ => quote! {
                        item
                    },
                });

                quote! {
                    #(#attrs)*
                    #unsafety impl< #generics > #trait_ for  #self_ty< #generics > {
                        #(#items)*
                    }
                }
            } else {
                quote! {
                    #input
                }
            },
            self_ty,
        )
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
        let default: TokenStream = params.default.value().parse()?;

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

#[derive(Debug)]
pub struct AvionicsParameters {
    default: LitStr,
    no_panic: bool,
}

impl Parse for AvionicsParameters {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.parse_terminated::<_, Token![,]>(ExprAssign::parse)?;

        let mut parsed = attributes
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

        let params = AvionicsParameters {
            default: parsed
                .remove("default")
                .ok_or_else(|| Error::new(attributes.span(), "missing required property `default`"))
                .and_then(|lit| {
                    if let Lit::Str(s) = lit {
                        Ok(s.clone())
                    } else {
                        Err(Error::new(
                            lit.span(),
                            "parameter `default` expects a string",
                        ))
                    }
                })?,
            no_panic: parsed
                .remove("no_panic")
                .map(|lit| {
                    if let Lit::Bool(b) = lit {
                        Ok(b.value)
                    } else {
                        Err(Error::new(
                            lit.span(),
                            "parameter `default` expects a boolean",
                        ))
                    }
                })
                .unwrap_or(Ok(true))?,
        };

        if !parsed.is_empty() {
            Err(Error::new(
                input.span(),
                format!(
                    "unexpected parameter(s): {}",
                    parsed.keys().cloned().collect::<Vec<String>>().join(", ")
                ),
            ))
        } else {
            Ok(params)
        }
    }
}
