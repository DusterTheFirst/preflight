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
    Check(CargoArguments),
    /// Run a set of automated tests on the project
    Test(CargoArguments),
    /// Run a simulation on the project
    Simulate(CargoArguments),
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
