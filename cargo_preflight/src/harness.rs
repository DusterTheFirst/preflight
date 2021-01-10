use core::panic::PanicInfo;
use std::{marker::PhantomData, mem::MaybeUninit, path::Path, sync::RwLock};

use dlopen::symbor::{Container, Ref, SymBorApi, Symbol};
use lazy_static::lazy_static;
use preflight::{
    abi::{AvionicsGuide, SetPanicCallback},
    Avionics, Control, Sensors,
};

use crate::{args::PanicHandleArguments, panic::panic_handle};

#[derive(SymBorApi)]
struct HarnessImpl<'a> {
    /// The callback into the avionics to request a control signal for guidance
    avionics_guide: Symbol<'a, AvionicsGuide>,
    /// A flag to ensure that the shared object was created with preflight
    #[dlopen_name = "__PREFLIGHT"]
    preflight: Ref<'a, bool>,
    /// Method to set the panic callback in order to be able to handle avionic panics
    set_panic_callback: Symbol<'a, SetPanicCallback>,
}

pub struct AvionicsHarness<P: AvionicsHarnessState> {
    harness: Container<HarnessImpl<'static>>,
    _panic: PhantomData<P>,
}

lazy_static! {
    static ref LAST_SENSORS: RwLock<Sensors> = RwLock::new(
        #[allow(unsafe_code, clippy::clippy::uninit_assumed_init)]
        // This is safe, since the sensors are always set before they are ever going to be read from
        // since a panic cannot happen until after this has been set at least once
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    );
}

pub struct PanicHang;
impl AvionicsHarnessState for PanicHang {}

pub struct PanicCaught;
impl AvionicsHarnessState for PanicCaught {}

pub trait AvionicsHarnessState {}

impl AvionicsHarness<PanicHang> {
    pub fn load(so: &Path) -> Result<Option<Self>, dlopen::Error> {
        #[allow(unsafe_code)]
        let harness: Container<HarnessImpl> = unsafe { Container::load(so) }?;

        if *harness.preflight {
            Ok(Some(AvionicsHarness {
                harness,
                _panic: PhantomData,
            }))
        } else {
            Ok(None)
        }
    }

    /// Setup panic handling for the guidance system using the given arguments
    pub fn setup_panic(self, args: PanicHandleArguments) -> AvionicsHarness<PanicCaught> {
        lazy_static! {
            static ref PANIC_ARGS: RwLock<PanicHandleArguments> = RwLock::new(Default::default());
        }

        *PANIC_ARGS.write().unwrap() = args;

        (self.harness.set_panic_callback)(|panic_info: &PanicInfo, avionics: &dyn Avionics| {
            panic_handle(
                panic_info,
                avionics,
                &LAST_SENSORS.read().unwrap(),
                &PANIC_ARGS.read().unwrap(),
            );
        });

        AvionicsHarness {
            _panic: PhantomData,
            harness: self.harness,
        }
    }
}

impl AvionicsHarness<PanicCaught> {
    /// Call into the avionics to request a guidance control signal given the inputted sensor data
    pub fn guide(&mut self, sensors: Sensors) -> Option<Control> {
        *LAST_SENSORS.write().unwrap() = sensors;

        (self.harness.avionics_guide)(&LAST_SENSORS.read().unwrap())
    }
}
