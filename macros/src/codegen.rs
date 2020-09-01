use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra};

mod assertions;
mod dispatchers;
mod hardware_tasks;
mod idle;
mod init;
mod locals;
mod module;
mod post_init;
mod pre_init;
mod resources;
mod resources_struct;
mod schedule;
mod schedule_body;
mod software_tasks;
mod spawn;
mod spawn_body;
mod timer_queue;
mod util;

// TODO document the syntax here or in `rtic-syntax`
pub fn app(app: &App, analysis: &Analysis, extra: &Extra) -> TokenStream2 {
    let mut const_app = vec![];
    let mut mains = vec![];
    let mut root = vec![];
    let mut user = vec![];

    // generate the `main` function
    let assertion_stmts = assertions::codegen(analysis);

    let pre_init_stmts = pre_init::codegen(&app, analysis, extra);

    let (const_app_init, root_init, user_init, call_init) = init::codegen(app, analysis, extra);

    let (const_app_post_init, post_init_stmts) = post_init::codegen(&app, analysis);

    let (const_app_idle, root_idle, user_idle, call_idle) = idle::codegen(app, analysis, extra);

    user.push(quote!(
        #user_init

        #user_idle
    ));

    root.push(quote!(
        #(#root_init)*

        #(#root_idle)*
    ));

    const_app.push(quote!(
        #const_app_init

        #(#const_app_post_init)*

        #const_app_idle
    ));

    let main = util::suffixed("main");
    let section = util::link_section("text");
    mains.push(quote!(
        #[no_mangle]
        #section
        unsafe extern "C" fn #main() -> ! {
            let _TODO: () = ();

            #(#assertion_stmts)*

            #(#pre_init_stmts)*

            #call_init

            #(#post_init_stmts)*

            #call_idle
        }
    ));

    let (const_app_resources, mod_resources) = resources::codegen(app, analysis, extra);

    let (const_app_hardware_tasks, root_hardware_tasks, user_hardware_tasks) =
        hardware_tasks::codegen(app, analysis, extra);

    let (const_app_software_tasks, root_software_tasks, user_software_tasks) =
        software_tasks::codegen(app, analysis, extra);

    let const_app_dispatchers = dispatchers::codegen(app, analysis, extra);

    let const_app_spawn = spawn::codegen(app, analysis, extra);

    let const_app_timer_queue = timer_queue::codegen(app, analysis, extra);

    let const_app_schedule = schedule::codegen(app, extra);

    let name = &app.name;
    let device = extra.device;
    quote!(
        #(#user)*

        #(#user_hardware_tasks)*

        #(#user_software_tasks)*

        #(#root)*

        #mod_resources

        #(#root_hardware_tasks)*

        #(#root_software_tasks)*

        /// Implementation details
        // the user can't access the items within this `const` item
        const #name: () = {
            /// Always include the device crate which contains the vector table
            use #device as _;

            #(#const_app)*

            #(#const_app_resources)*

            #(#const_app_hardware_tasks)*

            #(#const_app_software_tasks)*

            #(#const_app_dispatchers)*

            #(#const_app_spawn)*

            #(#const_app_timer_queue)*

            #(#const_app_schedule)*

            #(#mains)*
        };
    )
}
