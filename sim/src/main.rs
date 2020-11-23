use std::{num::Wrapping, ops::Range};

use iced::{
    button, image, pick_list, window, Align, Application, Button, Column, Command, Container,
    Element, Image, Length, PickList, Settings, Text,
};
use log::{info, LevelFilter};
use plotters::{
    prelude::LineSeries,
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea},
    style::BLACK,
    style::{IntoFont, RED, WHITE},
};
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use simplelog::{Config, TermLogger, TerminalMode};
use simulation::motor::{SupportedMotor, SUPPORTED_MOTORS};

mod simulation;

const GRAPH_WIDTH: u32 = 512;
const GRAPH_HEIGHT: u32 = 512;
const BUFFER_SIZE: usize = (GRAPH_HEIGHT * GRAPH_WIDTH * 4) as usize;

#[derive(Debug)]
struct Counter {
    pick_list: pick_list::State<SupportedMotor>,
    selected_motor: Option<SupportedMotor>,
    motor_thrust_curve: Box<[u8; BUFFER_SIZE]>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    MotorSelected(SupportedMotor),
}

impl Application for Counter {
    fn view(&mut self) -> Element<Message> {
        Column::new()
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(Text::new("Choose a motor from those bundled"))
            .push(PickList::new(
                &mut self.pick_list,
                SUPPORTED_MOTORS,
                self.selected_motor,
                Message::MotorSelected,
            ))
            .push(
                Image::new(image::Handle::from_pixels(
                    GRAPH_WIDTH,
                    GRAPH_HEIGHT,
                    /* include_bytes!("../test.png").to_vec(), */
                    self.motor_thrust_curve.to_vec(),
                ))
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .push(Text::new(
                self.motor_thrust_curve
                    .iter()
                    .cloned()
                    .fold(0u64, |a, b| a + b as u64)
                    .to_string(),
            ))
            .into()
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::MotorSelected(motor) => {
                self.selected_motor = Some(motor);

                let plot = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
                    &mut self.motor_thrust_curve[..],
                    (GRAPH_WIDTH, GRAPH_HEIGHT),
                )
                .unwrap()
                .into_drawing_area();

                plot.fill(&WHITE).unwrap();

                let low = motor.min.floor();
                let high = motor.max.ceil();

                let datapoints = ((low as i64)..(high as i64) * 100)
                    .map(|i| i as f64 * 0.01)
                    .map(|i| (i, (motor.thrust)(i).thrust));
                // let data =

                // After this point, we should be able to draw construct a chart context
                let mut chart = ChartBuilder::on(&plot)
                    // Set the caption of the chart
                    .caption("This is our first plot", ("sans-serif", 40).into_font())
                    // Set the size of the label region
                    .x_label_area_size(20)
                    .y_label_area_size(40)
                    // Finally attach a coordinate on the drawing area and make a chart context
                    .build_cartesian_2d(motor.min..motor.max, 0f64..10f64)
                    .unwrap();

                // Then we can draw a mesh
                chart
                    .configure_mesh()
                    // We can customize the maximum number of labels allowed for each axis
                    .x_labels(5)
                    .y_labels(5)
                    // We can also change the format of the label text
                    .y_label_formatter(&|x| format!("{:.3}", x))
                    .draw()
                    .unwrap();

                chart
                    .draw_series(LineSeries::new(datapoints, &RED))
                    .unwrap();

                info!("j");
            }
        }

        Command::none()
    }

    type Executor = iced::executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                motor_thrust_curve: Box::new([0; BUFFER_SIZE]),
                pick_list: pick_list::State::default(),
                selected_motor: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Cool?".into()
    }
}

fn main() -> iced::Result {
    TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed)
        .expect("Failed to initialize the logger");

    Counter::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../../fonts/Roboto/Roboto-Regular.ttf")),
        window: window::Settings {
            decorations: true,
            resizable: true,
            size: (1080, 512),
            always_on_top: true,
            ..Default::default()
        },
        ..Default::default()
    })
}
