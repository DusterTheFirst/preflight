use iced::{
    pick_list, svg, window, Align, Application, Column, Command, Element, Image, Length, PickList,
    Settings, Svg, Text,
};
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode};
use simulation::motor::{RocketMotor, SUPPORTED_MOTORS};
use ui::graph::Graph;

mod simulation;
mod ui;

struct Counter {
    pick_list: pick_list::State<RocketMotor>,
    selected_motor: Option<RocketMotor>,
    motor_thrust_curve: Graph,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    MotorSelected(RocketMotor),
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
                Image::new(self.motor_thrust_curve.as_handle())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            // .push(Text::new(
            //     self.motor_thrust_curve
            //         .pixels()
            //         .iter()
            //         .cloned()
            //         .fold(0u64, |a, b| a + b as u64)
            //         .to_string(),
            // ))
            .into()
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::MotorSelected(motor) => {
                self.selected_motor = Some(motor);

                self.motor_thrust_curve
                    .draw(motor)
                    .expect("Failed to render the motor graph");
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
                motor_thrust_curve: Graph::new(),
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
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Trace,
            ConfigBuilder::new().add_filter_allow_str("sim").build(),
            TerminalMode::Mixed,
        ),
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed),
    ])
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
