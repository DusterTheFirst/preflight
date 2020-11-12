use syn::{punctuated::Punctuated, PathSegment, Token};

pub fn reconstruct(segments: &Punctuated<PathSegment, Token![::]>) -> String {
    segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
