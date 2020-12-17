use core::panic::PanicInfo;
use std::{
    io::{self},
    panic, process, thread, unimplemented,
};

use cargo_preflight::{
    api::Api,
    args::{CargoArguments, CargoSpawnedArguments, PreflightCommand},
    cargo::{build_artifact, get_host_target, get_metadata},
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

    let CargoSpawnedArguments::Preflight(args) = CargoSpawnedArguments::from_args();

    match args.command {
        PreflightCommand::Check(args) => check(args, &mut shell),
        PreflightCommand::Test(_) => unimplemented!(),
        PreflightCommand::Simulate(_) => unimplemented!(),
    }
}

fn check(cargo_args: CargoArguments, shell: &mut Shell) -> io::Result<()> {
    let host_target = match get_host_target() {
        Ok(t) => t,
        Err(e) => {
            return shell.error(format!("{:#}", e));
        }
    };

    let metadata = match get_metadata(&cargo_args) {
        Err(e) => match e {
            cargo_metadata::Error::CargoMetadata { stderr } => {
                return shell.error(stderr.trim_start_matches("error: "))
            }
            e => return shell.error(e.to_string()),
        },
        Ok(metadata) => metadata,
    };

    if let Some(package) = metadata.root_package() {
        let has_dylib_target = package
            .targets
            .iter()
            .any(|t| t.kind.contains(&"dylib".to_string()));

        if has_dylib_target {
            shell.warning(
                "the crate probably should not have a library target with a crate_type of 'dylib'",
            )?;
            shell.note("this will be added automatically when this command is run. crate_type should be `staticlib` or `cdylib`")?;
        }

        match build_artifact(&cargo_args, &host_target, package) {
            Err(e) => return shell.error(format!("{:#}", e)),
            Ok(None) => {
                return shell.error("the cargo build did not produce any valid artifacts");
            }
            Ok(Some(artifact_file)) => {
                let api: Container<Api> = match unsafe { Container::load(artifact_file) } {
                    Ok(c) => c,
                    Err(e) => {
                        return shell.error(format!("failed to load built shared library: {}", e));
                    }
                };

                if *api.preflight() {
                    api.set_panic_callback(|panic_info: &PanicInfo| {
                        // shell.error(&format!("GUIDANCE SYSTEM PANIC!"));
                        println!("GUIDANCE SYSTEM PANIC WITH INPUT {}!\n{}", panic_info);
                        dbg!(thread::current());
                        process::exit(1);
                    });

                    dbg!(thread::current());

                    thread::Builder::new()
                        .name("flight control".into())
                        .spawn(move || {
                            let altitude = Length::new::<meter>(0.0);
                            let input = Sensors { altitude };
                            let result = api.avionics_guide(&input);
                            dbg!(&result);
                        })
                        .expect("Failed to spawn control thread")
                        .join()
                        .expect("Failed to complete control test");

                // if result.is_err() {
                //     shell.error(format!(
                //         "the avionics panicked with the input: {:?}",
                //         input
                //     ))?;
                // }
                } else {
                    return shell.error("the dylib was not setup using the `#[avionics_harness]` macro or is using an out of date dependency to preflight_impl");
                }
            }
        }
    } else {
        return shell.error("could not find the root package for this workspace");
    }

    Ok(())
}
