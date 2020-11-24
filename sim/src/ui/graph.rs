use crate::simulation::motor::RocketMotor;
use color_eyre::Help;
use iced::image::Handle;
use log::info;
use plotters::{
    coord::Shift,
    prelude::{
        ChartBuilder, DrawingArea, DrawingBackend, IntoDrawingArea, LineSeries, PointSeries,
        SVGBackend,
    },
    style::{Color, IntoFont, RGBAColor, ShapeStyle, RED, WHITE},
};
use plotters_bitmap::{bitmap_pixel::BGRXPixel, BitMapBackend};
use std::{
    pin::Pin,
    ptr::NonNull,
    sync::atomic::AtomicBool,
    sync::{atomic::Ordering, Arc, Once},
};

const GRAPH_WIDTH: u32 = 1024;
const GRAPH_HEIGHT: u32 = 1024;
const BUFFER_SIZE: usize = (GRAPH_HEIGHT * GRAPH_WIDTH * 4) as usize;

// type GraphBuf = [u8; BUFFER_SIZE];

static mut GRAPH_BUF: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
const IS_INIT: AtomicBool = AtomicBool::new(false);

pub struct Graph {
    backend: DrawingArea<BitMapBackend<'static, BGRXPixel>, Shift>,
}

impl Graph {
    pub fn new() -> Self {
        if !IS_INIT.load(Ordering::Relaxed) {
            IS_INIT.store(true, Ordering::Relaxed);

            Self {
                backend: BitMapBackend::with_buffer_and_format(
                    unsafe { &mut GRAPH_BUF },
                    (GRAPH_WIDTH, GRAPH_HEIGHT),
                )
                .unwrap()
                .into_drawing_area(),
            }
        } else {
            panic!("A graph was already constructed")
        }
    }

    pub fn as_handle(&mut self) -> Handle {
        // dbg!(unsafe { &GRAPH_BUF });
        Handle::from_pixels(GRAPH_WIDTH, GRAPH_HEIGHT, unsafe { GRAPH_BUF.to_vec() })
    }

    pub fn draw(&mut self, motor: RocketMotor) -> color_eyre::Result<()> {
        info!("a");
        let plot = &mut self.backend;

        plot.fill(&WHITE).note("Failed to draw the background")?;

        let datapoints = (motor.min.floor() as i64..=(motor.max * 100.0).ceil() as i64)
            .map(|i| i as f64 * 0.01)
            .map(|i| (i, (motor.thrust)(i).thrust))
            .collect::<Vec<_>>();

        // After this point, we should be able to draw construct a chart context
        let mut chart = ChartBuilder::on(&plot)
            // Set the caption of the chart
            .caption("This is our first plot", ("sans-serif", 40).into_font())
            // Set the size of the label region
            .x_label_area_size(20)
            .y_label_area_size(40)
            // Finally attach a coordinate on the drawing area and make a chart context
            .build_cartesian_2d(
                motor.min..motor.max,
                0f64..datapoints
                    .iter()
                    .map(|(_x, y)| y.ceil() as i64)
                    .max()
                    .unwrap_or_default() as f64,
            )
            .unwrap();

        // Then we can draw a mesh
        chart
            .configure_mesh()
            // We can customize the maximum number of labels allowed for each axis
            .x_labels(5)
            .y_labels(5)
            .label_style(("sans-serif", 80).into_font())
            // We can also change the format of the label text
            .y_label_formatter(&|x| format!("{:.3}", x))
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                datapoints,
                ShapeStyle {
                    color: RED.to_rgba(),
                    filled: false,
                    stroke_width: 20,
                },
            ))
            .unwrap()
            .label("H");

        plot.present().unwrap();

        info!("b");

        Ok(())
    }
}
