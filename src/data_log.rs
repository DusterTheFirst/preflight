use chrono::Local;
use std::{
    fs::{create_dir_all, File},
    io,
    path::Path,
    path::PathBuf,
    time::Instant,
    time::SystemTime,
};

pub struct DataLogger {
    file: File,
}

impl DataLogger {
    fn find_file() -> io::Result<File> {
        // Look in current directory for existing data dir
        let mut path = PathBuf::new();
        path.push("./data");

        if !path.exists() {
            create_dir_all(&path)?;
        }

        let local_time = Local::now();
        path.push(format!("{}", local_time.format("%Y-%m-%d-%H-%M-%S")));

        File::create(path)
    }

    pub fn new() -> io::Result<Self> {
        Ok(Self {
            file: Self::find_file()?,
        })
    }

    pub fn from_file(file: File) -> Self {
        Self { file }
    }
}
