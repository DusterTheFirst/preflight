use core::panic::PanicInfo;

use crate::{Control, Sensors};
use dlopen::wrapper::WrapperApi;

type PanicCallback = fn(panic_info: &PanicInfo);

#[derive(WrapperApi)]
pub struct Api<'a> {
    avionics_guide: fn(sensors: &Sensors) -> Option<Control>,
    #[dlopen_name = "__PREFLIGHT"]
    preflight: &'a bool,
    set_panic_callback: fn(callback: PanicCallback) -> Option<PanicCallback>,
}
