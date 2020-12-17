use core::panic::PanicInfo;
use std::{
    io::{self},
    panic, process,
    sync::{Arc, Mutex, RwLock},
    thread, unimplemented,
};

use anyhow::{anyhow, bail, ensure, Context};
use cargo_preflight::{
    api::Harness,
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
        PreflightCommand::Check(args) => {
            if let Err(e) = load_harness(args, &mut shell) {
                shell.error(format!("{:#}", e))?
            }
        }
        PreflightCommand::Test(_) => unimplemented!(),
        PreflightCommand::Simulate(_) => unimplemented!(),
    }

    Ok(())
}

fn run_harness(harness: Container<Harness<'static>>) {
    static LAST_SENSORS: Mutex<Sensors> = Mutex::new(Sensors {
        altitude: Length::new::<meter>(0.0),
    });

    harness.set_panic_callback(|panic_info: &PanicInfo| {
        // shell.error(&format!("GUIDANCE SYSTEM PANIC!"));
        println!(
            "GUIDANCE SYSTEM PANIC WITH INPUT {:?}!\n{}",
            LAST_SENSORS.lock().unwrap(),
            panic_info
        );
        process::exit(1);
    }); // TODO: THREAD TALKING FOR BETTER ERROR

    loop {
        let input = Sensors {
            altitude: Length::new::<meter>(0.0),
        };
        *LAST_SENSORS.lock().unwrap() = input;
        let result = harness.avionics_guide(&input);
        dbg!(&result);
    }
}

fn load_harness(
    cargo_args: CargoArguments,
    shell: &mut Shell,
) -> anyhow::Result<Arc<Container<Harness>>> {
    let host_target = get_host_target()?;

    let metadata = get_metadata(&cargo_args).map_err(|e| match e {
        cargo_metadata::Error::CargoMetadata { stderr } => {
            anyhow!("{}", stderr.trim_start_matches("error: "))
        }
        e => anyhow!("{}", e),
    })?;

    let package = metadata
        .root_package()
        .context("could not find the root package for this workspace")?;

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

    match build_artifact(&cargo_args, &host_target, package)? {
        None => {
            bail!("the cargo build did not produce any valid artifacts")
        }
        Some(artifact_file) => {
            shell.status("Loading", artifact_file.to_string_lossy())?;

            let harness: Container<Harness> = unsafe { Container::load(artifact_file) }
                .context("failed to load built shared library")?;

            ensure!(
                    *harness.preflight(),
                    "the dylib was not setup using the `#[avionics_harness]` macro or is using an out of date dependency to preflight_impl"
                );

            shell.status("Loaded", "");

            return Ok(Arc::new(harness));
        }
    }
}
