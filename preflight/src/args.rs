use std::path::PathBuf;
use structopt::{clap::AppSettings::ColoredHelp, StructOpt};

#[derive(StructOpt)]
#[structopt(global_setting(ColoredHelp))]
pub enum CargoSpawnedArguments {
    // TODO: sounds jank
    /// Run the preflight program
    Preflight(Arguments),
}

#[derive(StructOpt)]
pub struct Arguments {
    #[structopt(subcommand)]
    pub command: PreflightCommand,
}

#[derive(StructOpt)]
pub enum PreflightCommand {
    /// Check the layout and setup of the project for compatibility with the preflight runner
    Check {
        #[structopt(flatten)]
        cargo: CargoArguments,
    },
    /// Run a set of automated tests on the project
    Test {
        #[structopt(flatten)]
        cargo: CargoArguments,
        #[structopt(flatten)]
        panic: PanicHandleArguments,
        #[structopt(flatten)]
        sim: SimulationArguments,
        #[structopt(flatten)]
        display: DisplayArguments,
    },
    // /// Run a simulation on the project
    // Simulate {
    //     #[structopt(flatten)]
    //     cargo: CargoArguments,
    // },
}

#[derive(StructOpt)]
pub struct SimulationArguments {}

#[derive(StructOpt)]
pub struct DisplayArguments {
    /// Disable the gui for headless running
    #[structopt(long)]
    pub no_gui: bool
}

#[derive(StructOpt, Default)]
pub struct PanicHandleArguments {
    /// Open the panic report upon panic
    #[structopt(long, short)]
    pub open: bool,
}

#[derive(StructOpt)]
pub struct CargoArguments {
    #[structopt(long, parse(from_os_str), env = "CARGO")]
    pub cargo_path: PathBuf,
    // TODO: more
    /// Path to Cargo.toml
    #[structopt(long, name = "FILE", parse(from_os_str))]
    pub manifest_path: Option<PathBuf>,
    /// Run without accessing the network
    #[structopt(long)]
    pub offline: bool,
    /// Build artifacts in release mode, with optimizations
    #[structopt(long)]
    pub release: bool,
    /// Directory for all generated artifacts
    #[structopt(long, name = "DIRECTORY", parse(from_os_str))]
    pub target_dir: Option<PathBuf>,
}
