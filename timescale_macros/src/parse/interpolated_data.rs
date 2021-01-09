use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    AttrStyle, Attribute, Expr, ExprAssign, ExprLit, ExprPath, Ident, Lit, LitStr, Path,
    PathArguments, Result,
};

use crate::util::reconstruct;

/// Arguments to the `derive(InterpolatedData)` macro
#[derive(Debug)]
pub struct InterpolatedDataArgs {
    pub rename: LitStr,
}

impl Parse for InterpolatedDataArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            rename: input.parse()?,
        })
    }
}

impl InterpolatedDataArgs {
    pub fn parse_attributes(attrs: &[Attribute], field: &Ident) -> Result<Option<Self>> {
        Ok(attrs
            .iter()
            .filter(|attr| {
                // Filter all attributes with more than one path segment
                if let [segment] = attr.path.segments.iter().collect::<Vec<_>>().as_slice() {
                    // Ensure the attribute has no type arguments, is an outer
                    // attribute and also has the name timescale
                    attr.style == AttrStyle::Outer
                        && segment.arguments == PathArguments::None
                        && segment.ident.to_string() == "data"
                } else {
                    false
                }
            })
            .try_fold(None::<Self>, |pre, a| {
                let ExprAssign { left, right, .. }: ExprAssign = a.parse_args()?;

                if let Expr::Path(ExprPath {
                    path: Path { segments, .. },
                    ..
                }) = *left
                {
                    // Reconstruct the path of the attribute and match it by name
                    match reconstruct(&segments).as_str() {
                        "rename" => {
                            // Ensure a literal string is passed
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(str_lit),
                                ..
                            }) = *right
                            {
                                // Ensure no duplicates
                                if let Some(pre) = pre {
                                    Err(syn::Error::new(
                                        pre.rename.span(),
                                        format!("duplicate rename of `{}`", field.to_string()),
                                    ))
                                } else {
                                    Ok(Some(Self { rename: str_lit }))
                                }
                            } else {
                                Err(syn::Error::new(
                                    right.span(),
                                    "invalid expression, must be of format `rename = \"new_name\"`",
                                ))
                            }
                        }
                        name => Err(syn::Error::new(
                            segments.span(),
                            format!("unknown field `{}`", name),
                        )),
                    }
                } else {
                    Err(syn::Error::new(
                        right.span(),
                        "invalid expression, must be of format `field_name = field_value`",
                    ))
                }
            })?)
    }
}
