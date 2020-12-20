use core::panic::PanicInfo;

use crate::{Control, Sensors};
use dlopen::wrapper::WrapperApi;
use preflight_impl::Avionics;

type PanicCallback = fn(panic_info: &PanicInfo, avionics_state: &dyn Avionics);

#[derive(WrapperApi)]
pub struct Harness<'a> {
    /// The callback into the avionics to request a control signal for guidance
    avionics_guide: fn(sensors: &Sensors) -> Option<Control>,
    /// A flag to ensure that the shared object was created with preflight
    #[dlopen_name = "__PREFLIGHT"]
    preflight: &'a bool,
    /// Method to set the panic callback in order to be able to handle avionic panics
    set_panic_callback: fn(callback: PanicCallback) -> Option<PanicCallback>,
}
