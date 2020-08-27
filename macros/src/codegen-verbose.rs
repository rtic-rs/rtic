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
    let mut imports = vec![];

    // generate a `main` function for each core
    for core in 0..app.args.cores {
        let assertion_stmts = assertions::codegen(core, analysis, extra);

        let (const_app_pre_init, pre_init_stmts) = pre_init::codegen(core, &app, analysis, extra);

        let (const_app_init, _root_init, user_init, user_init_imports, call_init) =
            init::codegen(core, app, analysis, extra);

        let (const_app_post_init, post_init_stmts) =
            post_init::codegen(core, &app, analysis, extra);

        let (const_app_idle, _root_idle, user_idle, user_idle_imports, call_idle) =
            idle::codegen(core, app, analysis, extra);

        user.push(quote!(
            /// USER INIT
            #user_init

            /// USER IDLE
            #user_idle
        ));

        // Stow away the imports generated for each core
        imports.push(quote!(
            /// USER IMPORTS
            #(#user_init_imports)*

            /// USER IDLE
            #(#user_idle_imports)*
        ));

        root.push(quote!(
            #(#_root_init)*

            #(#_root_idle)*
        ));

        const_app.push(quote!(
            #(#const_app_pre_init)*

            #const_app_init

            #(#const_app_post_init)*

            #const_app_idle
        ));

        let cfg_core = util::cfg_core(core, app.args.cores);
        let main = util::suffixed("main", core);
        let section = util::link_section("text", core);
        mains.push(quote!(
            #[no_mangle]
            #section
            #cfg_core
            unsafe extern "C" fn #main() -> ! {
                #(#assertion_stmts)*

                #(#pre_init_stmts)*

                #call_init

                #(#post_init_stmts)*

                #call_idle
            }
        ));
    }

    let (const_app_resources, mod_resources, mod_resources_imports) =
        resources::codegen(app, analysis, extra);

    let (
        const_app_hardware_tasks,
        root_hardware_tasks,
        user_hardware_tasks,
        user_hardware_tasks_imports,
    ) = hardware_tasks::codegen(app, analysis, extra);

    let (
        const_app_software_tasks,
        root_software_tasks,
        user_software_tasks,
        user_software_tasks_imports,
    ) = software_tasks::codegen(app, analysis, extra);

    let const_app_dispatchers = dispatchers::codegen(app, analysis, extra);

    let const_app_spawn = spawn::codegen(app, analysis, extra);

    let const_app_timer_queue = timer_queue::codegen(app, analysis, extra);

    let const_app_schedule = schedule::codegen(app, extra);

    let cores = app.args.cores.to_string();
    let cfg_core = quote!(#[cfg(core = #cores)]);
    let msg = format!(
        "specified {} core{} but tried to compile for more than {0} core{1}",
        app.args.cores,
        if app.args.cores > 1 { "s" } else { "" }
    );
    let check_excess_cores = quote!(
        #cfg_core
        compile_error!(#msg);
    );

    /*
    for s in root.clone() {
        println!("{}", s.to_string());
    }
    */

    let user_imports = app.user_imports.clone();
    let user_code = app.user_code.clone();
    let name = &app.name;
    let device = extra.device;
    let endresult = quote!(
        /// USER
        #(#user)*

        /// USER_HW_TASKS
        #(#user_hardware_tasks)*

        /// USER_SW_TASKS
        #(#user_software_tasks)*

        /// ROOT
        //#(#root)*

        /// MOD_RESOURCES
        #mod_resources

        /// root_hardware_tasks
        #(#root_hardware_tasks)*

        /// root_software_tasks
        #(#root_software_tasks)*

        /// Implementation details
        mod #name {
            /// Always include the device crate which contains the vector table
            use #device as _;
            #(#imports)*
            /// User imports
            #(#user_imports)*

            /// User code from within the module
            #(#user_code)*

            /// User hardware tasks import
            #(#user_hardware_tasks_imports)*

            /// User software_tasks
            #(#user_software_tasks_imports)*

            /// Mod resources imports
            #(#mod_resources_imports)*

            #check_excess_cores

            /// Const app
            #(#const_app)*

            /// Const app resources
            #(#const_app_resources)*

            /// Const app hw tasks
            #(#const_app_hardware_tasks)*

            /// Const app sw tasks
            #(#const_app_software_tasks)*

            /// Const app dispatchers
            #(#const_app_dispatchers)*

            /// Const app spawn
            #(#const_app_spawn)*
            /// Const app spawn end

            #(#const_app_timer_queue)*

            #(#const_app_schedule)*

            /// Mains
            #(#mains)*
        }
    );
    for s in endresult.clone() {
        eprintln!("{}", s.to_string());
    }

    endresult
}
