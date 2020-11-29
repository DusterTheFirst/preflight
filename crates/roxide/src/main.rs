use application::Application;
use color_eyre::Help;
use gtk::Builder;

pub mod application;
pub mod graph;
pub mod simulation;

#[macro_use]
mod util;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    gtk::init()
        .note("Failed to initialize GTK")
        .suggestion("Ensure that GTK is installed and v3.24 or higher")?;

    let builder = Builder::from_string(include_str!("../glade/roxide.glade"));

    let app = Application::new(builder)?;

    app.show();

    gtk::main();

    Ok(())
}
