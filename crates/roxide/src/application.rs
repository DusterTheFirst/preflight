use std::{
    env, fs, process,
    rc::Rc,
    sync::atomic::AtomicBool,
    sync::{atomic::Ordering, Arc, Once, RwLock},
};

use color_eyre::Help;
use gtk::{
    ApplicationWindow, Builder, Button, ButtonExt, ComboBoxExt, ComboBoxText, ComboBoxTextExt,
    Inhibit, SpinButton, SpinButtonExt, SpinButtonSignals, WidgetExt,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{
    get_object, graph::GraphDisplay, simulation::motor::SUPPORTED_MOTORS, util::AtomicF64,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationState {
    pub selected_motor: RwLock<Option<usize>>,
    pub frequency: AtomicF64,
    // pub csv_log_folder: Option<PathBuf>, TODO:
    // pub csv_filename_override: Option<PathBuf>, TODO:
}

impl ApplicationState {
    fn new() -> Self {
        ApplicationState {
            selected_motor: RwLock::new(None),
            frequency: AtomicF64::new(60.0)
            // csv_filename_override: None,
            // csv_log_folder: Some(
            // env::current_dir().note("Failed to retrieve current working directory")?,
            // ),
        }
    }
}

lazy_static! {
    /// A static bool to suppress double updates from the timestep/frequency interconnection
    static ref SUPPRESS_UPDATE: AtomicBool = AtomicBool::new(false);
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
        let state = Arc::new(dbg!(Self::load_state()));

        Self {
            // Widgets
            application_window: get_object!(builder["application_window"]),
            simulation_frequency_input: get_object!(builder["simulation_frequency_input"]),
            simulation_timestep_input: get_object!(builder["simulation_timestep_input"]),
            motor_graph_button: get_object!(builder["motor_graph_button"]),
            motor_selector: get_object!(builder["motor_selector"]),

            // Custom Widgets
            graph_display: GraphDisplay::new(&builder, Arc::downgrade(&state))
                .note("Failed to load the graph window")?,

            // State
            state,
        }
        .load_state_into_application()?
        .setup_handlers()
    }

    fn save_state(state: &Arc<ApplicationState>) -> color_eyre::Result<()> {
        if let Some(mut cache_file) = dirs::cache_dir() {
            cache_file.push(format!("com.dusterthefirst.{}.ron", env!("CARGO_PKG_NAME")));

            fs::create_dir_all(&cache_file.parent().unwrap())
                .with_note(|| format!("Failed to create cache file: {:?}", cache_file))?;

            // Save the application state
            fs::write(
                &cache_file,
                ron::to_string(state.as_ref()).note("Failed to serialize the state")?,
            )
            .with_note(|| format!("Failed to write to cache file: {:?}", cache_file))?;
        } else {
            eprint!("User has no cache directory, discarding application state");
        }

        Ok(())
    }

    fn load_state() -> ApplicationState {
        if let Some(mut cache_file) = dirs::cache_dir() {
            cache_file.push(format!("com.dusterthefirst.{}.ron", env!("CARGO_PKG_NAME")));

            if cache_file.exists() {
                // Load the application state
                if let Ok(Ok(state)) = fs::read_to_string(&cache_file)
                    .with_note(|| format!("Failed to read from cache file: {:?}", cache_file))
                    .map(|x| ron::from_str(&x))
                {
                    return state;
                } else {
                    eprintln!("Failed to load in the state, falling back to default")
                }
            } else {
                eprintln!("Cache file does not exist, not attempting to load previous state");
            }
        } else {
            eprintln!(
                "User has no cache directory, not attempting to load previous application state"
            );
        }

        ApplicationState::new()
    }

    fn load_state_into_application(self) -> color_eyre::Result<Self> {
        // Load motors into the motor selector
        {
            self.motor_selector.append(Some("-1"), "None");

            self.motor_selector.set_active_id(Some("-1"));

            for (id, motor) in SUPPORTED_MOTORS.iter().enumerate() {
                self.motor_selector
                    .append(Some(&id.to_string()), motor.name)
            }
        }

        // Select the motor
        if let Some(id) = *self.state.selected_motor.read().unwrap() {
            self.motor_selector
                .set_active_id(Some(id.to_string().as_str()));
            self.motor_graph_button.set_sensitive(true);
        }

        // Load the timestamp and frequency into their fields
        let freq = self.state.frequency.load(Ordering::SeqCst);
        self.simulation_timestep_input
            .set_value((1.0 / freq) * 1000.0); // Timestep is in milliseconds
        self.simulation_frequency_input.set_value(freq);

        Ok(self)
    }

    fn setup_handlers(self) -> color_eyre::Result<Self> {
        ctrlc::set_handler(|| {
            static WARNING_MESSAGE: Once = Once::new();

            if WARNING_MESSAGE.is_completed() {
                eprintln!("Terminating");

                process::exit(1);
            }

            WARNING_MESSAGE.call_once(|| {
                eprintln!("\nThis is a UI application, please use the x button to safely close the program");
                eprintln!("Press Ctrl-C again to terminate");
            });
        }).note("Failed to set the Ctrl-C handler")?;

        self.application_window.connect_delete_event({
            let state = self.state.clone();

            move |_, _| {
                gtk::main_quit();

                Self::save_state(&state).unwrap();

                Inhibit(false)
            }
        });

        // Enforce the frequency as 1/timestep
        self.simulation_timestep_input.connect_value_changed({
            let freq_input = self.simulation_frequency_input.clone();
            let state = self.state.clone();

            move |simulation_timestep_input| {
                // Check if the update is suppressed.
                // This would be set if the update was caused by the frequency input adjusting the timestep to match
                if !SUPPRESS_UPDATE.load(Ordering::SeqCst) {
                    // Get the timestep and calculate the equivalent frequency
                    let timestep = simulation_timestep_input.get_value() / 1000.0;
                    let freq = 1.0 / timestep;

                    // Store the frequency in the application's state
                    state.frequency.store(freq, Ordering::SeqCst);

                    // Acquire a "lock" by suppressing the frequency handler and then update the displayed frequency
                    // discarding the lock afterwords
                    SUPPRESS_UPDATE.store(true, Ordering::SeqCst);
                    freq_input.set_value(freq);
                    SUPPRESS_UPDATE.store(false, Ordering::SeqCst);
                }
            }
        });

        // Enforce the timestep as 1/frequency
        self.simulation_frequency_input.connect_value_changed({
            let timestep_input = self.simulation_timestep_input.clone();
            let state = self.state.clone();

            move |simulation_frequency_input| {
                // See timestep handler for explanation
                if !SUPPRESS_UPDATE.load(Ordering::SeqCst) {
                    let freq = simulation_frequency_input.get_value();
                    let timestep = (1.0 / freq) * 1000.0;

                    state.frequency.store(freq, Ordering::SeqCst);

                    SUPPRESS_UPDATE.store(true, Ordering::SeqCst);
                    timestep_input.set_value(timestep);
                    SUPPRESS_UPDATE.store(false, Ordering::SeqCst);
                }
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

        Ok(self)
    }

    pub fn show(&self) {
        self.application_window.show_all();
    }
}
