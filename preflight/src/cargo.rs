use anyhow::{anyhow, bail, ensure, Context, Result};
use cargo_metadata::{Message, Metadata, MetadataCommand, Package};
use dlopen::utils::{PLATFORM_FILE_EXTENSION, PLATFORM_FILE_PREFIX};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use toml_edit::{value, Array, Document};

use crate::args::CargoArguments;

pub fn get_metadata(args: &CargoArguments) -> cargo_metadata::Result<Metadata> {
    let mut metadata_command = MetadataCommand::new();
    if let Some(manifest_path) = &args.manifest_path {
        metadata_command.manifest_path(manifest_path);
    }

    metadata_command.exec()
}

pub fn get_host_target() -> Result<String> {
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
) -> Result<Option<PathBuf>> {
    // Get the path to the projects manifest
    let manifest_path = cargo_args
        .manifest_path
        .as_ref()
        .unwrap_or(&package.manifest_path);

    // Create the path to a temporary manifest
    let temp_manifest_path = [
        &manifest_path
            .parent()
            .context("manifest has no parent directory")?,
        Path::new("target"),
        Path::new("preflight"),
        Path::new("~Cargo.toml.bak"),
    ]
    .iter()
    .collect::<PathBuf>();

    // Ensure the parent directory exists
    {
        let dir = temp_manifest_path.parent().unwrap();
        fs::create_dir_all(dir).with_context(|| format!("failed to create dir {:?}", dir))?;
    }

    {
        // Copy the user's manifest to the temporary manifest for backup
        fs::copy(&manifest_path, &temp_manifest_path).with_context(|| {
            format!(
                "failed to copy manifest from {:?} to temporary file {:?}",
                manifest_path, temp_manifest_path
            )
        })?;

        // Load in the user's manifest
        let manifest = fs::read_to_string(manifest_path)
            .with_context(|| format!("failed to read manifest from {:?}", manifest_path))?;

        // Parse it for editing
        let mut doc = manifest
            .parse::<Document>()
            .with_context(|| format!("failed to parse manifest at {:?}", manifest_path))?;

        // Edit the crate-type
        doc["lib"]["crate-type"] = value({
            let mut a = Array::default();

            a.push("dylib").unwrap();

            a
        });

        // Save the edited manifest back to the Cargo.toml
        fs::write(&manifest_path, doc.to_string_in_original_order())
            .with_context(|| format!("failed to write manifest to {:?}", temp_manifest_path))?;
    }

    // Build the program
    let mut build_command = Command::new(&cargo_args.cargo_path)
        .args(
            [
                "rustc",
                "--message-format=json-render-diagnostics",
                &format!("--target={}", target_override),
                &format!("--manifest-path={}", manifest_path.to_string_lossy()),
                if cargo_args.offline { "--offline" } else { "" },
                if cargo_args.release { "--release" } else { "" },
            ]
            .iter()
            .filter(|x| !x.is_empty()),
        )
        .env("__PREFLIGHT", "testing")
        .stdout(Stdio::piped())
        .spawn()
        .context("`cargo` failed to run to completion")?;

    // Create a vector to hold the found artifacts' paths
    let mut artifacts = Vec::with_capacity(4);

    // Read in the artifacts
    let reader = BufReader::new(build_command.stdout.take().unwrap());
    for message in Message::parse_stream(reader) {
        match message.context("failed to read output from `cargo`")? {
            // Only react to dylib artifacts for this crate
            Message::CompilerArtifact(artifact)
                if artifact.package_id == package.id
                    && artifact.target.kind.contains(&"dylib".to_string()) =>
            {
                // Find the file that matches the dylib that is produced
                let new_file = artifact.filenames.into_iter().find(|file| {
                    file.file_name().map_or(false, |name| {
                        let name = name.to_string_lossy();

                        name.starts_with(PLATFORM_FILE_PREFIX)
                            && name.ends_with(PLATFORM_FILE_EXTENSION)
                    })
                });

                // Add the artifact if it exists
                if let Some(f) = new_file {
                    artifacts.push(f);
                }
            }
            _ => (),
        }
    }

    // Get the one artifact if there is only one, if there are multiple, error out
    let artifact = match artifacts.len() {
        0 | 1 => artifacts.pop(),
        _ => bail!("found ambiguous dylib artifacts: {:?}", artifacts),
    };

    // Wait for cargo to finish
    let status = build_command
        .wait()
        .context("`cargo` failed to run to completion")?;

    // Revert manifest
    {
        fs::copy(&temp_manifest_path, &manifest_path).with_context(|| {
            format!(
                "failed to copy old manifest from {:?} back to {:?}",
                temp_manifest_path, manifest_path
            )
        })?;

        fs::remove_file(&temp_manifest_path).with_context(|| {
            format!(
                "failed to delete the temporary manifest at {:?}",
                temp_manifest_path
            )
        })?;
    }

    ensure!(
        status.success(),
        "`rustc` exited with a failure code [{0}]",
        status
    );

    Ok(artifact)
}
