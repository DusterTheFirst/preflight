use core::panic::PanicInfo;

use dlopen::wrapper::WrapperApi;
use preflight_impl::Avionics;

type PanicCallback = fn(panic_info: &PanicInfo);

#[derive(WrapperApi)]
pub struct Harness<'a> {
    /// A flag to ensure that the shared object was created with preflight
    #[dlopen_name = "__PREFLIGHT"]
    preflight: &'a bool,
    /// Get a reference to the current avionics's state as a debug representation
    get_avionics_state: fn() -> &'a dyn Avionics,
    /// Get a mutable reference to the current avionics's state as a debug representation
    get_avionics_state_mut: fn() -> &'a mut dyn Avionics,
    /// Method to set the panic callback in order to be able to handle avionic panics
    set_panic_callback: fn(callback: PanicCallback) -> Option<PanicCallback>,
}
