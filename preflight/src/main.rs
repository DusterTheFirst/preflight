use core::panic::PanicInfo;
use std::{
    io::{self},
    panic, process,
    sync::Arc,
    sync::RwLock,
    unimplemented,
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use cargo_preflight::{
    api::Harness,
    args::{CargoArguments, CargoSpawnedArguments, PreflightCommand},
    cargo::{build_artifact, get_host_target, get_metadata},
    shell::Shell,
    Vector3,
};
use dlopen::wrapper::Container;
use lazy_static::lazy_static;
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
            } else {
                shell.status("Success", "built and loaded avionics harness successfully")?;
            }
        }
        PreflightCommand::Test(args) => match load_harness(args, &mut shell) {
            Err(e) => shell.error(format!("{:#}", e))?,
            Ok(harness) => match fuzz_harness(harness) {
                Err(e) => shell.error(format!("{:#}", e))?,
                Ok(false) => shell.error("harness failed to run")?,
                Ok(true) => shell.status("Finished", "TODO:")?,
            },
        },
        PreflightCommand::Simulate(_) => unimplemented!(),
    }

    Ok(())
}

fn fuzz_harness(harness: Container<Harness<'static>>) -> Result<bool> {
    lazy_static! {
        static ref LAST_SENSORS: RwLock<Sensors> = RwLock::new(Sensors {
            altitude: Length::new::<meter>(0.0),
            linear_acceleration: Vector3::zero(),
            gravity_acceleration: Vector3::zero(),
            both_acceleration: Vector3::zero(),
            orientation: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            magnetic_field: Vector3::zero(),
        });
        static ref AVIONICS_HARNESS: Arc<RwLock<Option<Container<Harness<'static>>>>> =
            Arc::new(RwLock::new(None));
    }

    *AVIONICS_HARNESS.write().unwrap() = Some(harness);

    let harness = AVIONICS_HARNESS.clone();
    let harness = harness.read().unwrap();
    let harness = harness.as_ref().unwrap(); // TODO: CLEAN UP?

    harness.set_panic_callback(|panic_info: &PanicInfo| {
        println!(
            "\nGUIDANCE SYSTEM PANIC!\n{}\n----INPUT----\n{:#?}\n----CURRENT STATE----\n{:#?}",
            panic_info,
            *LAST_SENSORS.read().unwrap(),
            AVIONICS_HARNESS
                .clone()
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .get_avionics_state() // TODO: Clean up?
        );
        process::exit(1);
    });

    for _ in 0..10 {
        *LAST_SENSORS.write().unwrap() = Sensors {
            altitude: Length::new::<meter>(0.0),
            linear_acceleration: Vector3::zero(),
            gravity_acceleration: Vector3::zero(),
            both_acceleration: Vector3::zero(),
            orientation: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            magnetic_field: Vector3::zero(),
        };

        // println!(
        //     "{}",
        //     Length::format_args(
        //         Length::new::<meter>(0.0),
        //         cargo_preflight::uom::fmt::DisplayStyle::Description
        //     )
        // );

        let result = harness.avionics_guide(&LAST_SENSORS.read().unwrap());

        dbg!(&result);
    }

    Ok(false)
}

fn load_harness(
    cargo_args: CargoArguments,
    shell: &mut Shell,
) -> anyhow::Result<Container<Harness<'static>>> {
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

            return Ok(harness);
        }
    }
}
