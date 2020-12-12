use std::{
    error::Error,
    fmt,
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
};

use fmt::Formatter;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// Little macro to help get an object from a builder and propagate the error
#[macro_export]
macro_rules! get_object {
    ($builder:ident[$name:literal]) => {{
        use color_eyre::{eyre::eyre, Help};
        use gtk::prelude::BuilderExtManual;

        $builder
            .get_object($name)
            .ok_or(eyre!(concat!(
                "Object with id `",
                $name,
                "` missing from the glade builder"
            )))
            .suggestion("Check the spelling at the location above and in the glade file")?
    }};
}

pub struct AtomicF64(AtomicU64);
impl AtomicF64 {
    pub fn new(val: f64) -> Self {
        Self(AtomicU64::new(val.to_bits()))
    }

    pub fn load(&self, order: Ordering) -> f64 {
        f64::from_bits(self.0.load(order))
    }

    pub fn store(&self, val: f64, order: Ordering) {
        self.0.store(val.to_bits(), order)
    }
}

impl Debug for AtomicF64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.load(Ordering::SeqCst))
    }
}

impl Serialize for AtomicF64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.load(Ordering::SeqCst))
    }
}

struct AtomicF64Visitor;

impl<'de> Visitor<'de> for AtomicF64Visitor {
    type Value = AtomicF64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a 64 bit floating point number")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(AtomicF64::new(v))
    }
}

impl<'de> Deserialize<'de> for AtomicF64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(deserializer.deserialize_f64(AtomicF64Visitor)?)
    }
}
