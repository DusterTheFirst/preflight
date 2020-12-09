use cargo_metadata::{Message, Metadata, MetadataCommand, Package};
use dlopen::utils::{PLATFORM_FILE_EXTENSION, PLATFORM_FILE_PREFIX};
use std::{
    ffi::OsStr,
    io::{self, BufReader},
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
};
use thiserror::Error;

use crate::args::CargoArguments;

pub fn get_metadata(args: &CargoArguments) -> cargo_metadata::Result<Metadata> {
    let mut metadata_command = MetadataCommand::new();
    if let Some(manifest_path) = &args.manifest_path {
        metadata_command.manifest_path(manifest_path);
    }

    metadata_command.exec()
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("found ambiguous dylib artifacts: {0:?}")]
    AmbiguousArtifacts(Vec<PathBuf>),
    #[error("`cargo` failed to run to completion: {0}")]
    CargoSpawnError(#[from] io::Error),
    #[error("build failed [{0}]")]
    CargoCompileFail(ExitStatus),
}

pub fn build_artifact<P: AsRef<OsStr>>(
    cargo_path: &P,
    package: &Package,
) -> Result<Option<PathBuf>, BuildError> {
    let mut build_command = Command::new(cargo_path)
        .args(&[
            "build",
            "--message-format=json-render-diagnostics",
            &format!(
                "--manifest-path={}",
                package.manifest_path.to_string_lossy()
            ),
        ])
        .env("__PREFLIGHT", "testing")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut artifacts = Vec::with_capacity(4);

    let reader = BufReader::new(build_command.stdout.take().unwrap());
    for message in Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerArtifact(artifact) => {
                if artifact.package_id == package.id
                    && artifact.target.kind.contains(&"dylib".to_string())
                {
                    let new_file = artifact.filenames.into_iter().find(|file| {
                        file.file_name().map_or(false, |name| {
                            let name = name.to_string_lossy();

                            name.starts_with(PLATFORM_FILE_PREFIX)
                                && name.ends_with(PLATFORM_FILE_EXTENSION)
                        })
                    });

                    match new_file {
                        None => (),
                        Some(f) => {
                            artifacts.push(f);
                        }
                    }
                }
            }
            _ => (), // Unknown message
        }
    }

    let artifact = match artifacts.len() {
        0 | 1 => artifacts.pop(),
        _ => return Err(BuildError::AmbiguousArtifacts(artifacts)),
    };

    let status = build_command.wait()?;

    if status.success() {
        Ok(artifact)
    } else {
        Err(BuildError::CargoCompileFail(status))
    }
}
