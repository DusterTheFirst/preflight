use std::{
    io::{self, BufReader},
    panic,
    path::PathBuf,
    process::{Command, Stdio},
    unimplemented,
};

use cargo_preflight::{
    api::Api,
    args::{Arguments, PreflightCheckArguments, PreflightCommand},
    cargo::{build_artifact, get_metadata},
    shell::Shell,
};
use dlopen::wrapper::Container;
use preflight_impl::{
    uom::si::length::{meter, Length},
    Sensors,
};
use structopt::StructOpt;

fn main() -> io::Result<()> {
    let mut shell = Shell::new();

    let args: Arguments = Arguments::from_args();

    match args.command {
        PreflightCommand::Check(args) => check(args, &mut shell),
        PreflightCommand::Simulate(_) => unimplemented!(),
    }
}

fn check(args: PreflightCheckArguments, shell: &mut Shell) -> io::Result<()> {
    let metadata = match get_metadata(&args.cargo) {
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
            match build_artifact(&args.cargo.cargo_path, package) {
                Err(e) => return shell.error(e.to_string()),
                Ok(None) => {
                    return shell.error("the cargo build did not produce any valid artifacts");
                }
                Ok(Some(artifact_file)) => {
                    let api: Container<Api> = match unsafe { Container::load(artifact_file) } {
                        Ok(c) => c,
                        Err(e) => {
                            return shell
                                .error(format!("failed to load built shared library: {}", e));
                        }
                    };

                    if *api.preflight() {
                        let altitude = Length::new::<meter>(0.0);
                        let input = Sensors { altitude };
                        let result = panic::catch_unwind(|| api.avionics_guide(&input));
                        dbg!(&result);
                        if result.is_err() {
                            shell.error(format!(
                                "the avionics panicked with the input: {:?}",
                                input
                            ))?;
                        }
                    } else {
                        return shell.error("the dylib was not setup using the `#[avionics_harness]` macro or is using an out of date dependency to preflight_impl");
                    }
                }
            }
        } else {
            return shell
                .error("the crate must have a library target with a crate_type of 'dylib'");
        }
    } else {
        return shell.error("could not find the root package for this workspace");
    }

    Ok(())
}
