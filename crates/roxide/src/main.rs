use std::{process, sync::Once};

use application::Application;
use color_eyre::Help;
use gtk::Builder;
use log::{LevelFilter, info, trace, warn};
use simplelog::{CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode};

pub mod application;
pub mod simulation;

#[macro_use]
mod util;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    CombinedLogger::init(vec![
        TermLogger::new(
            if cfg!(debug_assertions) {
                LevelFilter::Trace
            } else {
                LevelFilter::Info
            },
            ConfigBuilder::new()
                .add_filter_allow_str(env!("CARGO_PKG_NAME"))
                .build(),
            TerminalMode::Mixed,
        ),
        TermLogger::new(
            if cfg!(debug_assertions) {
                LevelFilter::Info
            } else {
                LevelFilter::Warn
            },
            ConfigBuilder::new()
                .add_filter_ignore_str(env!("CARGO_PKG_NAME"))
                .build(),
            TerminalMode::Mixed,
        ),
    ])
    .note("Failed to initialize the logger")
    .suggestion("Make sure you are running this in a tty")?;

    ctrlc::set_handler(|| {
        static WARNING_MESSAGE: Once = Once::new();

        if WARNING_MESSAGE.is_completed() {
            trace!("Terminating");

            process::exit(1);
        }

        WARNING_MESSAGE.call_once(|| {
            warn!("This is a UI application, please use the x button to safely close the program");
            info!("Press Ctrl-C again to terminate");
        });
    })
    .note("Failed to set the Ctrl-C handler")?;

    gtk::init()
        .note("Failed to initialize GTK")
        .suggestion("Ensure that GTK is installed and v3.24 or higher")?;

    let builder = Builder::from_string(include_str!("../glade/roxide.glade"));

    let app = Application::new(builder)?;

    app.show();

    gtk::main();

    Ok(())
}
