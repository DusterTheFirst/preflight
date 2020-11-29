use std::{
    rc::Rc,
    sync::{Arc, RwLock, Weak},
};

use cairo::Context;
use color_eyre::{eyre::eyre, Help};
use gtk::{
    get_current_event_time, prelude::BuilderExtManual, ApplicationWindow, Builder, Button,
    ButtonExt, ComboBoxExt, ComboBoxText, ComboBoxTextExt, DrawingArea, GtkWindowExt, Inhibit,
    SpinButton, SpinButtonExt, SpinButtonSignals, WidgetExt, Window,
};
use plotters::{
    prelude::{ChartBuilder, IntoDrawingArea, LineSeries, PathElement},
    style::{Color, BLACK, RED, WHITE},
};
use plotters_cairo::CairoBackend;
use simulation::motor::{RocketMotor, SUPPORTED_MOTORS};

pub mod simulation;

/// Little macro to help get an object from a builder and propagate the error
macro_rules! get_object {
    ($builder:ident[$name:literal]) => {
        $builder
            .get_object($name)
            .ok_or(eyre!(concat!(
                "Object with id `",
                $name,
                "` missing from the glade builder"
            )))
            .suggestion("Check the spelling at the location above and in the glade file")?;
    };
}

struct GraphDisplay {
    window: Window,

    // Widgets
    drawing_area: DrawingArea,

    // State
    state: Weak<ApplicationState>,
}

impl GraphDisplay {
    pub fn new(builder: &Builder, state: Weak<ApplicationState>) -> color_eyre::Result<Rc<Self>> {
        Ok(Rc::new(Self {
            window: get_object!(builder["graph_window"]),
            drawing_area: get_object!(builder["graph_drawing_area"]),
            state,
        })
        .setup())
    }

    pub fn setup(self: Rc<Self>) -> Rc<Self> {
        let this = self.clone();
        self.drawing_area
            .connect_draw(move |graph_drawing_area, cr| {
                if let Some(motor) = this
                    .state
                    .upgrade()
                    .map(|s| *s.selected_motor.read().unwrap())
                    .flatten()
                {
                    let (allocation, _) = graph_drawing_area.get_allocated_size();
                    let size = (allocation.width as u32 - 20, allocation.height as u32 - 20);

                    this.draw(cr, size, motor);
                } else {
                    // Hide the window if there is no selected motor
                    this.window.hide();
                }

                Inhibit(false)
            });

        self.window.connect_delete_event(|window, _| {
            window.hide();

            Inhibit(true)
        });

        self
    }

    fn draw(&self, cr: &Context, size: (u32, u32), motor: RocketMotor) {
        let root = CairoBackend::new(cr, size).unwrap().into_drawing_area();

        root.fill(&WHITE).unwrap();

        let datapoints = (motor.min.floor() as i64..=(motor.max * 100.0).ceil() as i64)
            .map(|i| i as f64 * 0.01)
            .map(|i| (i, (motor.thrust)(i).thrust))
            .collect::<Vec<_>>();

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .margin(10)
            .build_cartesian_2d(
                motor.min..motor.max,
                0f64..datapoints
                    .iter()
                    .map(|(_x, y)| y.ceil() as i64)
                    .max()
                    .unwrap_or_default() as f64,
            )
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(LineSeries::new(datapoints, &RED))
            .unwrap()
            .label(motor.name)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()
            .unwrap();
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

struct ApplicationState {
    selected_motor: RwLock<Option<RocketMotor>>,
}

struct Application {
    // Widgets
    application_window: ApplicationWindow,
    simulation_timestep_input: SpinButton,
    simulation_frequency_input: SpinButton,
    motor_graph_button: Button,
    motor_selector: ComboBoxText,

    // Custom Widgets
    graph_display: Rc<GraphDisplay>,

    // State
    state: Arc<ApplicationState>,
}

impl Application {
    pub fn new(builder: Builder) -> color_eyre::Result<Self> {
        let state = Arc::new(ApplicationState {
            selected_motor: RwLock::new(None),
        });

        Ok(Self {
            // Widgets
            application_window: get_object!(builder["application_window"]),
            simulation_frequency_input: get_object!(builder["simulation_timestep_input"]),
            simulation_timestep_input: get_object!(builder["simulation_frequency_input"]),
            motor_graph_button: get_object!(builder["motor_graph_button"]),
            motor_selector: get_object!(builder["motor_selector"]),

            // Custom Widgets
            graph_display: GraphDisplay::new(&builder, Arc::downgrade(&state))
                .note("Failed to load the graph window")?,

            // State
            state,
        }
        .setup())
    }

    pub fn setup(self) -> Self {
        self.application_window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        // Enforce the frequency as 1/timestep
        self.simulation_timestep_input.connect_value_changed({
            let freq_input = self.simulation_frequency_input.clone();

            move |simulation_timestep_input| {
                freq_input.set_value(1.0 / simulation_timestep_input.get_value())
            }
        });

        // Enforce the timestep as 1/frequency
        self.simulation_frequency_input.connect_value_changed({
            let timestep_input = self.simulation_timestep_input.clone();

            move |simulation_frequency_input| {
                timestep_input.set_value(1.0 / simulation_frequency_input.get_value())
            }
        });

        self.motor_graph_button.connect_clicked({
            let display = self.graph_display.clone();

            move |_| display.show()
        });

        self.motor_selector.connect_changed({
            let state = self.state.clone();
            let button = self.motor_graph_button.clone();
            let display = self.graph_display.clone();

            move |motor_selector| {
                let mut motor = state.selected_motor.write().unwrap();

                if let Some(id) = motor_selector.get_active_id() {
                    if let Ok(id) = id.parse::<usize>() {
                        *motor = Some(SUPPORTED_MOTORS[id]);

                        button.set_sensitive(true);
                        display.queue_draw();
                    } else {
                        *motor = None;

                        button.set_sensitive(false);
                        display.hide();
                    }
                } else {
                    *motor = None;

                    button.set_sensitive(false);
                }
            }
        });
        self.motor_selector.append(Some("-1"), "None");

        self.motor_selector.set_active_id(Some("-1"));

        for (id, motor) in SUPPORTED_MOTORS.iter().enumerate() {
            self.motor_selector
                .append(Some(&id.to_string()), motor.name)
        }

        self
    }

    pub fn show(&self) {
        self.application_window.show_all();
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    gtk::init()
        .note("Failed to initialize GTK")
        .suggestion("Ensure that GTK is installed and v3.24 or higher")?;

    let builder = Builder::from_string(include_str!("../glade/roxide.glade"));

    let app = Application::new(builder)?;

    app.show();

    gtk::main();

    Ok(())
}
