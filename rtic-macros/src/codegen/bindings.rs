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
pub fn interrupt_mod_ident(app: &crate::codegen::App) -> proc_macro2::TokenStream {
    let device = &app.args.device;

    let span = proc_macro2::Span::call_site();
    let enum_ = syn::Ident::new("interrupt", span);
    quote::quote!(#device::enum_)
}
