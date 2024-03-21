use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::analyze::Analysis;
use crate::syntax::ast::App;

pub mod bindings;

mod assertions;
mod async_dispatchers;
mod extra_mods;
mod hardware_tasks;
mod idle;
mod init;
mod local_resources;
mod local_resources_struct;
mod module;
mod post_init;
mod pre_init;
mod shared_resources;
mod shared_resources_struct;
mod software_tasks;
mod util;

mod main;

// TODO: organize codegen to actual parts of code
// so `main::codegen` generates ALL the code for `fn main`,
// `software_tasks::codegen` generates ALL the code for software tasks etc...

#[allow(clippy::too_many_lines)]
pub fn app(app: &App, analysis: &Analysis) -> TokenStream2 {
    // Generate the `main` function
    let main = main::codegen(app, analysis);
    let init_codegen = init::codegen(app, analysis);
    let idle_codegen = idle::codegen(app, analysis);
    let shared_resources_codegen = shared_resources::codegen(app, analysis);
    let local_resources_codegen = local_resources::codegen(app, analysis);
    let hardware_tasks_codegen = hardware_tasks::codegen(app, analysis);
    let software_tasks_codegen = software_tasks::codegen(app, analysis);
    let async_dispatchers_codegen = async_dispatchers::codegen(app, analysis);

    let user_imports = &app.user_imports;
    let user_code = &app.user_code;
    let name = &app.name;
    let device = &app.args.device;

    let rt_err = util::rt_err_ident();
    let async_limit = bindings::async_prio_limit(app, analysis);

    quote!(
        /// The RTIC application module
        pub mod #name {
            /// Always include the device crate which contains the vector table
            use #device as #rt_err;

            #(#async_limit)*

            #(#user_imports)*

            #(#user_code)*
            /// User code end

            #init_codegen

            #idle_codegen

            #hardware_tasks_codegen

            #software_tasks_codegen

            #shared_resources_codegen

            #local_resources_codegen

            #async_dispatchers_codegen

            #main
        }
    )
}
