use crate::{Sensors, Control};
use dlopen::wrapper::WrapperApi;

#[derive(WrapperApi)]
pub struct Api<'a> {
    avionics_guide: fn(sensors: &Sensors) -> Option<Control>,
    #[dlopen_name = "__PREFLIGHT"]
    preflight: &'a bool,
}