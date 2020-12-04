use std::{
    env,
    io::{self, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};

use cargo_metadata::{Artifact, Message, Metadata, MetadataCommand};
use dlopen::utils::{PLATFORM_FILE_EXTENSION, PLATFORM_FILE_PREFIX};
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

fn main() -> io::Result<()> {
    let mut shell = Shell::new();

    let cargo_exec = match env::var("CARGO") {
        Err(_) => return shell.error("program must be invoked through cargo"),
        Ok(c) => c,
    };

    let args = Arguments::from_args();

    let mut metadata_command = MetadataCommand::new();
    if let Some(manifest_path) = args.manifest_path {
        metadata_command.manifest_path(manifest_path);
    }

    let metadata = match metadata_command.exec() {
        Err(e) => match e {
            cargo_metadata::Error::CargoMetadata { stderr } => {
                return shell.error(stderr.trim_start_matches("error: "))
            }
            e => return shell.error(e.to_string()),
        },
        Ok(metadata) => metadata,
    };

    if let Some(package) = metadata.root_package() {
        if package
            .targets
            .iter()
            .any(|t| t.kind.contains(&"dylib".to_string()))
        {
            let mut build_command = Command::new(cargo_exec)
                .args(&["build", "--message-format=json-render-diagnostics"])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let reader = BufReader::new(build_command.stdout.take().unwrap());
            for message in Message::parse_stream(reader) {
                match message.unwrap() {
                    Message::CompilerArtifact(artifact) => {
                        if artifact.package_id == package.id
                            && artifact.target.kind.contains(&"dylib".to_string())
                        {
                            let artifact_file = artifact.filenames.into_iter().find(|file| {
                                file.file_name().map_or(false, |name| {
                                    let name = name.to_string_lossy();

                                    name.starts_with(PLATFORM_FILE_PREFIX)
                                        && name.ends_with(PLATFORM_FILE_EXTENSION)
                                })
                            });

                            dbg!(artifact_file);
                        }
                    }
                    _ => (), // Unknown message
                }
            }

            if build_command
                .wait()
                .expect("Couldn't get cargo's exit status")
                .success()
            {
            } else {
                return shell.error("cargo failed to build");
            }
        } else {
            return shell
                .error("the crate must have a library target with a crate_type of 'dylib'");
        }
    } else {
        return shell.error("could not find the root package for this workspace");
    }

    shell.status("Metadata", &format!("{:#?}", 0))?;

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

    Ok(())
}
