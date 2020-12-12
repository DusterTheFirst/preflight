use std::{
    fs,
    sync::{Arc, RwLock},
};

use color_eyre::Help;
use log::{trace, warn};
use serde::{Deserialize, Serialize};

use crate::util::AtomicF64;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationState {
    pub selected_motor: RwLock<Option<usize>>,
    pub frequency: AtomicF64,
    // pub csv_log_folder: Option<PathBuf>,
    // pub csv_filename_override: Option<PathBuf>, TODO:
}

impl ApplicationState {
    fn new() -> Self {
        ApplicationState {
            selected_motor: RwLock::new(None),
            frequency: AtomicF64::new(60.0)
            // csv_filename_override: None,
            // csv_log_folder: Some(
            // env::current_dir().note("Failed to retrieve current working directory")?,
            // ),
        }
    }

    pub fn save(self: &Arc<ApplicationState>) -> color_eyre::Result<()> {
        if let Some(mut cache_file) = dirs::cache_dir() {
            cache_file.push(format!("com.dusterthefirst.{}.ron", env!("CARGO_PKG_NAME")));

            fs::create_dir_all(&cache_file.parent().unwrap())
                .with_note(|| format!("Failed to create cache file: {:?}", cache_file))?;

            // Save the application state
            fs::write(
                &cache_file,
                ron::to_string(self.as_ref()).note("Failed to serialize the state")?,
            )
            .with_note(|| format!("Failed to write to cache file: {:?}", cache_file))?;
        } else {
            trace!("User has no cache directory, discarding application state");
        }

        Ok(())
    }

    pub fn load() -> ApplicationState {
        if let Some(mut cache_file) = dirs::cache_dir() {
            cache_file.push(format!("com.dusterthefirst.{}.ron", env!("CARGO_PKG_NAME")));

            if cache_file.exists() {
                // Load the application state
                match fs::read_to_string(&cache_file) {
                    Ok(contents) => match ron::from_str(&contents) {
                        Ok(state) => return state,
                        Err(e) => warn!("Cache file contained invalid data, assuming corruption or schema update, falling back to default state: {}", e)
                    },
                    Err(e) => warn!(
                        "Failed to read from cache file `{:?}`, falling back to default state: {}",
                        cache_file, e
                    ),
                }
            } else {
                trace!("Cache file does not exist, not attempting to load previous state");
            }
        } else {
            trace!(
                "User has no cache directory, not attempting to load previous application state"
            );
        }

        ApplicationState::new()
    }
}
