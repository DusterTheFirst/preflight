use std::{any::Any, sync::Arc};

pub trait Fields {
    fn field_names(&self) -> Option<&'static [&'static str]> {
        None
    }
    fn field_values(&self) -> Option<&'static [Arc<dyn Any>]> {
        None
    }
}

impl Fields for f64 {}
impl Fields for f32 {}

impl Fields for u8 {}
impl Fields for u16 {}
impl Fields for u32 {}
impl Fields for u64 {}
impl Fields for usize {}

impl Fields for i8 {}
impl Fields for i16 {}
impl Fields for i32 {}
impl Fields for i64 {}
impl Fields for isize {}


#[allow(unused_imports)]
#[macro_use]
extern crate fields_derive;

#[doc(hidden)]
pub use fields_derive::*;
