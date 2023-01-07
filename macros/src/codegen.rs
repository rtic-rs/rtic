use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::analyze::Analysis;
use crate::syntax::ast::App;

mod assertions;
mod async_dispatchers;
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

    let (mod_app_init, root_init, user_init) = init::codegen(app, analysis);

    let (mod_app_idle, root_idle, user_idle) = idle::codegen(app, analysis);

    let (mod_app_shared_resources, mod_shared_resources) = shared_resources::codegen(app, analysis);
    let (mod_app_local_resources, mod_local_resources) = local_resources::codegen(app, analysis);

    let (mod_app_hardware_tasks, root_hardware_tasks, user_hardware_tasks) =
        hardware_tasks::codegen(app, analysis);

    let (mod_app_software_tasks, root_software_tasks, user_software_tasks) =
        software_tasks::codegen(app, analysis);

    let mod_app_async_dispatchers = async_dispatchers::codegen(app, analysis);
    let user_imports = &app.user_imports;
    let user_code = &app.user_code;
    let name = &app.name;
    let device = &app.args.device;

    let rt_err = util::rt_err_ident();

    quote!(
        /// The RTIC application module
        pub mod #name {
            /// Always include the device crate which contains the vector table
            use #device as #rt_err;

            #(#user_imports)*

            /// User code from within the module
            #(#user_code)*
            /// User code end

            #(#user_hardware_tasks)*

            #(#user_software_tasks)*

            #mod_app_init

            #(#root_init)*

            #user_init

            #(#mod_app_idle)*

            #(#root_idle)*

            #user_idle

            #mod_shared_resources

            #mod_local_resources

            #(#root_hardware_tasks)*

            #(#root_software_tasks)*

            #(#mod_app_shared_resources)*

            #(#mod_app_local_resources)*

            #(#mod_app_hardware_tasks)*

            #(#mod_app_software_tasks)*

            #(#mod_app_async_dispatchers)*

            #main
        }
    )
}
