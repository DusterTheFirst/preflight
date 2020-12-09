use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Arguments {
    pub command_name: String,
    #[structopt(subcommand)]
    pub command: PreflightCommand,
}

#[derive(StructOpt)]
pub enum PreflightCommand {
    Check(PreflightCheckArguments),
    Simulate(PreflightSimulateArguments),
}

#[derive(StructOpt)]
pub struct PreflightCheckArguments {
    #[structopt(flatten)]
    pub cargo: CargoArguments,
}

#[derive(StructOpt)]
pub struct PreflightSimulateArguments {
    #[structopt(flatten)]
    pub cargo: CargoArguments,
}

#[derive(StructOpt)]
pub struct CargoArguments {
    #[structopt(long, parse(from_os_str), env = "CARGO")]
    pub cargo_path: PathBuf,
    // TODO: more
    #[structopt(long, name = "FILE", parse(from_os_str))]
    pub manifest_path: Option<PathBuf>,
}
