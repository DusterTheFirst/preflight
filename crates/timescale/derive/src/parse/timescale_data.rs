use syn::{
    parse::{Parse, ParseStream},
    AttrStyle, Attribute, Ident, LitStr, PathArguments, Result,
};

/// Arguments to the `derive(TimescaleData)` macro
#[derive(Debug)]
pub struct FieldArgs {
    pub rename: LitStr,
}

impl Parse for FieldArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldArgs {
            rename: input.parse()?,
        })
    }
}

impl FieldArgs {
    pub fn parse_field_attributes(attrs: &[Attribute], field: &Ident) -> Result<Option<Self>> {
        Ok(attrs
            .iter()
            .filter(|attr| {
                // Filter all attributes with more than one path segment
                if let [segment] = attr.path.segments.iter().collect::<Vec<_>>().as_slice() {
                    // Ensure the attribute has no type arguments, is an outer
                    // attribute and also has the name rename
                    attr.style == AttrStyle::Outer
                        && segment.arguments == PathArguments::None
                        && segment.ident.to_string() == "rename"
                } else {
                    false
                }
            })
            .try_fold(None::<FieldArgs>, |pre, a| {
                let input: LitStr = a.parse_args()?;

                if let Some(pre) = pre {
                    Err(syn::Error::new(
                        pre.rename.span(),
                        format!("duplicate rename of field `{}`", field.to_string()),
                    ))
                } else {
                    Ok(Some(FieldArgs { rename: input }))
                }
            })?)
    }
}
