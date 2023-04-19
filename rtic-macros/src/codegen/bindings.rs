use crate::codegen::{util, App};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[cfg(not(any(
    feature = "cortex-m-source-masking",
    feature = "cortex-m-basepri",
    feature = "riscv-slic",
    feature = "test-template",
)))]
compile_error!("No backend selected");

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
pub use cortex::*;

#[cfg(feature = "riscv-slic")]
pub use self::riscv_slic::*;

#[cfg(feature = "test-template")]
pub use template::*;

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
mod cortex;

#[cfg(feature = "riscv-slic")]
mod riscv_slic;

#[cfg(feature = "test-template")]
mod template;

#[cfg(not(feature = "riscv-slic"))]
/// Utility function to get the device identifier as to refer to the interrupt enum.
/// In most of the cases, this is just [`App::args::device`].
/// However, the SLIC implementation is slightly different, as it creates its own enum.
pub fn interrupt_mod_ident(app: &App) -> TokenStream2 {
    let device = &app.args.device;
    let enum_ = util::interrupt_ident();
    quote!(#device::enum_)
}
