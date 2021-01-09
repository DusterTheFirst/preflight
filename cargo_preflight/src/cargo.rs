use anyhow::{anyhow, bail, ensure, Context};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use dlopen::utils::{PLATFORM_FILE_EXTENSION, PLATFORM_FILE_PREFIX};
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::args::CargoArguments;

pub fn get_metadata(args: &CargoArguments) -> cargo_metadata::Result<Metadata> {
    let mut metadata_command = MetadataCommand::new();
    if let Some(manifest_path) = &args.manifest_path {
        metadata_command.manifest_path(manifest_path);
    }

    metadata_command.exec()
}

pub fn get_host_target() -> anyhow::Result<String> {
    let output = Command::new("rustc")
        .args(&["-Vv"])
        .stdout(Stdio::piped())
        .output()
        .context("failed to spawn `rustc`")?;

    ensure!(
        output.status.success(),
        "`rustc` exited with a failure code [{0}]",
        output.status
    );

    let mut reader = BufReader::new(&output.stdout[..]);
    let mut string = String::new();

    while reader.read_line(&mut string)? != 0 {
        if let Some(triple) = string.strip_prefix("host: ").map(|s| s.trim()) {
            return Ok(triple.to_string());
        }

        string.clear();
    }

    Err(anyhow!(
        "unable to find host target triple in rustc output: \n{}",
        textwrap::indent(
            &String::from_utf8(output.stdout)
                .context("failed to parse in output from `rustc` as UTF-8")?,
            "    "
        )
    ))
}

pub fn build_artifact<'a>(
    cargo_args: &'a CargoArguments,
    target_override: &str,
    package: &'a Package,
) -> anyhow::Result<Option<PathBuf>> {
    // Build the program
    let mut build_command = Command::new(&cargo_args.cargo_path)
        .args(
            [
                "rustc",
                &format!("--target={}", target_override),
                if cargo_args.offline { "--offline" } else { "" },
                if cargo_args.release { "--release" } else { "" },
                "--",
                "--crate-type=dylib",
            ]
            .iter()
            .filter(|x| !x.is_empty()),
        )
        .env("__PREFLIGHT", "testing")
        .spawn()
        .context("`cargo` failed to run to completion")?;

    // Wait for cargo to finish
    let status = build_command
        .wait()
        .context("`cargo` failed to run to completion")?;

    // Get the dylib artifacts
    let mut artifacts = glob::glob(&format!(
        "target/{target}/{profile}/deps/{prefix}{package}*.{ext}",
        target = target_override,
        profile = if cargo_args.release {
            "release"
        } else {
            "debug"
        },
        prefix = PLATFORM_FILE_PREFIX,
        package = package.name,
        ext = PLATFORM_FILE_EXTENSION
    ))
    .context("Failed to read glob pattern")?
    .collect::<Result<Vec<_>, _>>()
    .context("Failed to get artifact path")?;

    // Get the one artifact if there is only one, if there are multiple, error out
    let artifact = match artifacts.len() {
        0 | 1 => artifacts.pop(),
        _ => bail!("found ambiguous dylib artifacts: {:?}", artifacts),
    };

    ensure!(
        status.success(),
        "`rustc` exited with a failure code [{0}]",
        status
    );

    Ok(artifact)
}
