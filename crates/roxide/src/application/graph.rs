use std::{
    f64::NAN,
    iter,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    sync::Weak,
};

use cairo::Context;
use color_eyre::Help;
use gtk::{
    get_current_event_time, Builder, DrawingArea, GtkWindowExt, Inhibit, ToggleToolButton,
    ToggleToolButtonExt, ToolButtonExt, WidgetExt, Window,
};
use plotters::{
    prelude::{ChartBuilder, IntoDrawingArea, LineSeries, PathElement},
    style::{Color, Palette, Palette99, BLACK, WHITE},
};
use plotters_cairo::CairoBackend;

use crate::{
    application::ApplicationState, get_object, simulation::motor::RocketMotor,
    simulation::motor::SUPPORTED_MOTORS,
};

pub struct GraphDisplay {
    window: Window,

    // Widgets
    drawing_area: DrawingArea,
    show_all_motors_button: ToggleToolButton,

    // State
    state: Weak<ApplicationState>,
    show_all: Rc<AtomicBool>,
}

impl GraphDisplay {
    pub fn new(builder: &Builder, state: Weak<ApplicationState>) -> color_eyre::Result<Rc<Self>> {
        Ok(Rc::new(Self {
            window: get_object!(builder["graph_window"]),
            drawing_area: get_object!(builder["graph_drawing_area"]),
            show_all_motors_button: get_object!(builder["show_all_motors_button"]),
            state,
            show_all: Rc::new(AtomicBool::new(false)),
        })
        .setup())
    }

    pub fn setup(self: Rc<Self>) -> Rc<Self> {
        self.drawing_area.connect_draw({
            let this = self.clone();

            move |graph_drawing_area, cr| {
                if let Some(motor) = this
                    .state
                    .upgrade()
                    .map(|s| {
                        *s.selected_motor
                            .read()
                            .expect("Failed to read the selected motor")
                    })
                    .flatten()
                {
                    let (allocation, _) = graph_drawing_area.get_allocated_size();
                    let size = (allocation.width as u32 - 20, allocation.height as u32 - 20);

                    this.draw(cr, size, motor).unwrap();
                } else {
                    // Hide the window if there is no selected motor
                    this.window.hide();
                }

                Inhibit(false)
            }
        });

        self.show_all_motors_button.connect_clicked({
            let show_all = self.show_all.clone();
            let drawing_area = self.drawing_area.clone();

            move |show_all_motors_button| {
                show_all.store(show_all_motors_button.get_active(), Ordering::Relaxed);
                drawing_area.queue_draw();
            }
        });

        self.window.connect_delete_event(|window, _| {
            window.hide();

            Inhibit(true)
        });

        self
    }

    fn draw(&self, cr: &Context, size: (u32, u32), highlight: usize) -> color_eyre::Result<()> {
        let motors: Box<dyn Iterator<Item = (usize, &RocketMotor)>> =
            if self.show_all.load(Ordering::Relaxed) {
                Box::new(SUPPORTED_MOTORS.iter().enumerate())
            } else {
                Box::new(iter::once((highlight, &SUPPORTED_MOTORS[highlight])))
            };

        let root = CairoBackend::new(cr, size)
            .note("Failed to create the cairo backend")?
            .into_drawing_area();

        root.fill(&WHITE).note("Failed to fill the background")?;

        let motor_datapoints = motors
            .into_iter()
            .map(|(id, motor)| {
                (
                    id,
                    motor,
                    (motor.min.floor() as i64..=(motor.max * 100.0).ceil() as i64)
                        .map(|i| i as f64 * 0.01)
                        .map(|i| (i, (motor.thrust)(i).thrust))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        let min = motor_datapoints
            .iter()
            .map(|(_, motor, _)| motor.min)
            .fold(NAN, f64::min);

        let max = motor_datapoints
            .iter()
            .map(|(_, motor, _)| motor.max)
            .fold(NAN, f64::max);

        let peak = motor_datapoints
            .iter()
            .map(|(_, _, datapoints)| datapoints.iter().map(|(_x, y)| *y).fold(NAN, f64::max))
            .fold(NAN, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .margin(10)
            .build_cartesian_2d(min..max, 0f64..peak)
            .note("Failed to build the coordinate plane")?;

        chart
            .configure_mesh()
            .draw()
            .note("Failed drawing the coordinate mesh")?;

        for (i, motor, datapoints) in motor_datapoints {
            let style = Palette99::pick(i).stroke_width(if i == highlight { 3 } else { 1 });

            chart
                .draw_series(LineSeries::new(datapoints, style.clone()))
                .note("Failed in drawing the motor's series")?
                .label(motor.name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], style.clone()));
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()
            .note("Failed in drawing the legend")?;

        Ok(())
    }

    pub fn show(&self) {
        self.window.present_with_time(get_current_event_time())
    }

    pub fn hide(&self) {
        self.window.hide()
    }

    pub fn queue_draw(&self) {
        self.drawing_area.queue_draw();
    }
}
