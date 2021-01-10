use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Error, ItemImpl, Result, spanned::Spanned};

#[derive(Debug, FromMeta)]
pub struct AvionicsParameters {
    default: String,
    #[darling(default)]
    no_panic: bool,
}

pub fn harness(params: AvionicsParameters, input: ItemImpl) -> Result<TokenStream> {
    let (implementation, st) = {
        let ItemImpl {
            self_ty, trait_, ..
        } = &input;

        let (invert, trait_, _) = &trait_
            .as_ref()
            .ok_or_else(|| Error::new(input.span(), "no trait was found to implement"))?;


        if !trait_.is_ident("Avionics") {
            return Err(Error::new(
                trait_.span(),
                "expected a trait implementation of `Avionics`",
            ));
        }

        if let Some(invert) = invert {
            return Err(Error::new(
                invert.span(),
                "cannot negate the `Avionics` implementation",
            ));
        }

        (&input, self_ty)
    };

    let default = {
        let default: TokenStream = params.default.parse()?;

        quote_spanned! {params.default.span()=>
            #default
        }
    };

    let (avionics_impl, panic_impl) = if std::env::var_os("__PREFLIGHT").is_some() {
        (
            quote! {
                use preflight::abi::*;

                #[no_mangle]
                pub static __PREFLIGHT: bool = cfg!(preflight);

                #[no_mangle]
                pub static avionics_guide: AvionicsGuide = |sensors: &Sensors| unsafe {
                    AVIONICS.guide(sensors)
                };

                static mut __PANIC_CALLBACK: Option<PanicCallback> = None;

                #[no_mangle]
                pub static set_panic_callback: SetPanicCallback = |callback: PanicCallback| unsafe {
                    __PANIC_CALLBACK.replace(callback);
                };
            },
            quote! {
                if let Some(callback) = unsafe { __PANIC_CALLBACK } {
                    callback(_panic_info, unsafe { &AVIONICS })
                }
            },
        )
    } else {
        (
            quote! {
                unsafe extern "C" fn avionics_guide(sensors: &Sensors) -> Option<Control> {
                    AVIONICS.guide(sensors)
                }
            },
            quote! {
                extern "C" {
                    fn panic_abort();
                }

                unsafe { panic_abort() };
            },
        )
    };

    let panic_handler = if params.no_panic {
        quote! {}
    } else {
        quote! {
            // TODO: PUT uC IN DEEP SLEEP ON PANIC OR SMTHN or call back into c code to handle panic
            #[cfg(not(any(test, trybuild)))]
            #[panic_handler]
            fn handle_panic(_panic_info: &core::panic::PanicInfo) -> ! {
                #panic_impl

                loop {
                    core::sync::atomic::spin_loop_hint()
                }
            }
        }
    };

    Ok(quote! {
        #implementation

        #[doc(hidden)]
        mod __PREFLIGHT {
            use super::*;

            #[allow(unused)]
            static mut AVIONICS: #st = #default();

            #avionics_impl

            #panic_handler
        }
    })
}
