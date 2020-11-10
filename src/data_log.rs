use chrono::Local;
use csv::{Writer, WriterBuilder};
use std::{
    fs::{create_dir_all, File},
    io,
    marker::PhantomData,
    path::PathBuf,
};
use timescale::ToTimescale;

pub struct DataLogger<Datapoint: ToTimescale> {
    writer: Writer<File>,
    data: PhantomData<Datapoint>,
}

impl<Datapoint: ToTimescale> DataLogger<Datapoint> {
    fn find_file() -> io::Result<File> {
        // Look in current directory for existing data dir
        let mut path = PathBuf::new();
        path.push("./data");

        if !path.exists() {
            create_dir_all(&path)?;
        }

        let local_time = Local::now();
        path.push(format!("{}.csv", local_time.format("%Y-%m-%d-%H-%M-%S")));

        File::create(path)
    }

    pub fn new() -> io::Result<Self> {
        Ok(Self::from_file(Self::find_file()?))
    }

    pub fn from_file(file: File) -> Self {
        Self {
            writer: WriterBuilder::new().from_writer(file),
            data: PhantomData,
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    pub fn add_data_point(&mut self, time: f64, data_point: Datapoint) -> csv::Result<()> {
        self.writer.serialize(data_point.with_time(time))
    }
}
