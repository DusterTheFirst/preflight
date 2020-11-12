use crate::util::reconstruct;
use syn::{
    parse::Parse, parse::ParseStream, punctuated::Punctuated, spanned::Spanned, AttrStyle,
    Attribute, Expr, ExprAssign, ExprLit, ExprPath, Lit, LitStr, Path, PathArguments, Token,
};

/// Arguments to the `derive(TimescaleDataTable)` macro
#[derive(Default, Debug)]
pub struct StructArgs {
    pub st: Option<Path>,
    pub file: Option<LitStr>,
}

impl Parse for StructArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the arguments in as an array of expressions
        let arguments: Punctuated<ExprAssign, Token![,]> =
            input.parse_terminated(ExprAssign::parse)?;

        Ok(arguments
            .into_iter()
            .try_fold(Self::default(), |pre, expr| {
                // Make sure the left side of the expression is a path
                if let Expr::Path(ExprPath { path, .. } ) = *expr.left {
                    // Join the path together
                    let name = reconstruct(&path.segments);

                    // Match the argument by name
                    match name.as_str() {
                        "st" => {
                            // Expect the right side to be a path to some structure
                            if let Expr::Path(ExprPath { path , .. }) = *expr.right {
                                // Ensure the struct was not defined earlier
                                if pre.st.is_some() {
                                    Err(syn::Error::new(
                                        path.span(),
                                        "duplicate definition of field `st`",
                                    ))
                                } else {
                                    Ok(Self {
                                        st: Some(path),
                                        ..pre
                                    })
                                }
                            } else {
                                Err(syn::Error::new(
                                    path.span(),
                                    "invalid argument syntax.\nexpected syntax `st = NameOfStruct`",
                                ))
                            }
                        },
                        "file" => {
                            // Expect the right side to be a path to some string literal
                            if let Expr::Lit(ExprLit { lit: Lit::Str(str_lit), .. }) = *expr.right {
                                // Ensure the file was not defined earlier
                                if pre.file.is_some() {
                                    Err(syn::Error::new(
                                        str_lit.span(),
                                        "duplicate definition of field `file`",
                                    ))
                                } else {
                                    Ok(Self {
                                        file: Some(str_lit),
                                        ..pre
                                    })
                                }
                            } else {
                                Err(syn::Error::new(
                                    path.span(),
                                    "invalid argument syntax.\nexpected syntax `file = \"path/to/file.csv\"`",
                                ))
                            }
                        },
                        name => Err(syn::Error::new(
                            path.segments.span(),
                            format!("unknown argument `{}`.\navailable arguments are `file` and `st`", name),
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        expr.span(),
                        "invalid argument syntax.\nexpected syntax `argument_name = argument_value`",
                    ))
                }
            })?)
    }
}

impl StructArgs {
    pub fn parse_attributes(attributes: &[Attribute]) -> syn::Result<Self> {
        attributes
            .iter()
            .filter(|attr| {
                // Filter all attributes with more than one path segment
                if let [segment] = attr.path.segments.iter().collect::<Vec<_>>().as_slice() {
                    // Ensure the attribute has no type arguments, is an outer
                    // attribute and also has the name csv
                    attr.style == AttrStyle::Outer
                        && segment.arguments == PathArguments::None
                        && segment.ident.to_string() == "csv"
                } else {
                    false
                }
            })
            .try_fold(StructArgs::default(), |pre, a| {
                // Parse the args from the attribute
                let input = a.parse_args::<StructArgs>()?;

                // Ensure no duplicate definitions between attributes
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
                    Ok(Self {
                        file: input.file.or(pre.file),
                        st: input.st.or(pre.st),
                    })
                }
            })
    }
}
