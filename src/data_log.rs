use chrono::Local;
use csv::Writer;
use fields::{Fields, SerializeFlatten};
use serde::{ser::SerializeStruct, Serialize};
use std::{
    fmt::Debug,
    fs::{create_dir_all, File},
    io,
    marker::PhantomData,
    path::PathBuf,
};

#[derive(Debug, Fields, SerializeFlatten)]
struct TimescaleData<Datapoint: Clone + Debug + Serialize> {
    time: f64,
    data_point: Datapoint,
}

pub struct DataLogger<Datapoint: Clone + Fields + Debug + Serialize> {
    writer: Writer<File>,
    data: PhantomData<TimescaleData<Datapoint>>,
}

impl<Datapoint: Clone + Fields + Debug + Serialize> DataLogger<Datapoint> {
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
            writer: Writer::from_writer(file),
            data: PhantomData,
        }
    }

    pub fn add_data_point(&mut self, time: f64, data_point: Datapoint) -> csv::Result<()> {
        let data = dbg!(TimescaleData { time, data_point });

        dbg!(serde_json::to_string(&data));

        self.writer.serialize(data)
    }
}
