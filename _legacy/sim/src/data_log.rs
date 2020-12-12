use chrono::Local;
use csv::Writer;
use std::{
    fs::{create_dir_all, File},
    io,
    marker::PhantomData,
    path::PathBuf,
};
use timescale::ToTimescale;

pub trait DataLogger<Datapoint> {
    fn flush(&mut self) -> io::Result<()>;
    fn add_data_point(&mut self, time: f64, data_point: Datapoint) -> io::Result<()>;
    fn enable(&mut self);
    fn disable(&mut self);
}

pub struct CsvLogger<Datapoint: ToTimescale> {
    writer: Writer<File>,
    data: PhantomData<Datapoint>,
    enabled: bool,
}

impl<Datapoint: ToTimescale> CsvLogger<Datapoint> {
    pub fn new(purpose: &str, enabled: bool) -> io::Result<Self> {
        // Look in current directory for existing data dir
        let mut path = PathBuf::new();
        path.push("simulation_data");

        let local_time = Local::now();
        path.push(purpose);

        if !path.exists() {
            create_dir_all(&path)?;
        }

        path.push(format!("{}.csv", local_time.format("%Y-%m-%d-%H-%M-%S")));

        Ok(Self {
            data: PhantomData,
            enabled,
            writer: Writer::from_path(path)?,
        })
    }
}

impl<Datapoint: ToTimescale> DataLogger<Datapoint> for CsvLogger<Datapoint> {
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn add_data_point(&mut self, time: f64, data_point: Datapoint) -> io::Result<()> {
        if self.enabled {
            self.writer
                .serialize(data_point.with_time(time))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        } else {
            Ok(())
        }
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false;
    }
}

pub struct NopLogger;

impl<Datapoint: ToTimescale> DataLogger<Datapoint> for NopLogger {
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn add_data_point(&mut self, _: f64, _: Datapoint) -> io::Result<()> {
        Ok(())
    }
    fn enable(&mut self) {}
    fn disable(&mut self) {}
}
