use std::{env, io, path::PathBuf};

use cargo_metadata::{Metadata, MetadataCommand};
use color_eyre::{eyre::Context, Help};
use env::VarError;
use fly::Shell;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Arguments {
    pub command_name: String,
    // TODO: more
    #[structopt(long, name = "FILE", parse(from_os_str))]
    pub manifest_path: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Arguments::from_args();

    let mut shell = Shell::new();

    let mut metadata_command = MetadataCommand::new();
    if let Some(manifest_path) = args.manifest_path {
        metadata_command.manifest_path(manifest_path);
    }
    let metadata = metadata_command
        .exec()
        .note("Failed to get the metadata about the crate using cargo")?;

    shell.status("Metadata", &format!("{:#?}", metadata.root_package()))?;

    shell.status("Arguments", &format!("{:#?}", std::env::args()))?;
    // shell.status(
    //     "Variables",
    //     &format!(
    //         "{}",
    //         std::env::vars()
    //             .map(|(key, val)| format!("\n{:12}: {}", key, val))
    //             .collect::<String>()
    //     ),
    // )?;

    dbg!(env::var("CARGO")
        .wrap_err("Failed to get environment variable CARGO")
        .note("Ensure that this command is run with cargo")?);

    Ok(())
}
