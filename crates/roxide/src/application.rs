use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use color_eyre::Help;
use gtk::{
    ApplicationWindow, Builder, Button, ButtonExt, ComboBoxExt, ComboBoxText, ComboBoxTextExt,
    Inhibit, SpinButton, SpinButtonExt, SpinButtonSignals, WidgetExt,
};

use crate::{get_object, graph::GraphDisplay, simulation::motor::SUPPORTED_MOTORS};

pub struct ApplicationState {
    pub selected_motor: RwLock<Option<usize>>,
}

pub struct Application {
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
                freq_input.set_value(1.0 / (simulation_timestep_input.get_value() / 1000.0))
            }
        });

        // Enforce the timestep as 1/frequency
        self.simulation_frequency_input.connect_value_changed({
            let timestep_input = self.simulation_timestep_input.clone();

            move |simulation_frequency_input| {
                timestep_input.set_value((1.0 / simulation_frequency_input.get_value()) * 1000.0)
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
                let mut motor = state
                    .selected_motor
                    .write()
                    .expect("Failed to read the selected motor");

                if let Some(id) = motor_selector.get_active_id() {
                    if let Ok(id) = id.parse::<usize>() {
                        *motor = Some(id);

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
