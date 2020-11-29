/// Little macro to help get an object from a builder and propagate the error
#[macro_export]
macro_rules! get_object {
    ($builder:ident[$name:literal]) => {{
        use gtk::prelude::BuilderExtManual;
        use color_eyre::{Help, eyre::eyre};

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
