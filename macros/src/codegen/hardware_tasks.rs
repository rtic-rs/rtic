use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{locals, module, resources_struct, util},
};

/// Generate support code for hardware tasks (`#[exception]`s and `#[interrupt]`s)
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // const_app_hardware_tasks -- interrupt handlers and `${task}Resources` constructors
    Vec<TokenStream2>,
    // root_hardware_tasks -- items that must be placed in the root of the crate:
    // - `${task}Locals` structs
    // - `${task}Resources` structs
    // - `${task}` modules
    Vec<TokenStream2>,
    // user_hardware_tasks -- the `#[task]` functions written by the user
    Vec<TokenStream2>,
) {
    let mut const_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];

    for (name, task) in &app.hardware_tasks {
        let core = task.args.core;
        let cfg_core = util::cfg_core(core, app.args.cores);

        let (let_instant, instant) = if app.uses_schedule(core) {
            let m = extra.monotonic();

            (
                Some(quote!(let instant = <#m as rtfm::Monotonic>::now();)),
                Some(quote!(, instant)),
            )
        } else {
            (None, None)
        };

        let locals_new = if task.locals.is_empty() {
            quote!()
        } else {
            quote!(#name::Locals::new(),)
        };

        let symbol = if cfg!(feature = "homogeneous") {
            util::suffixed(&task.args.binds.to_string(), core)
        } else {
            task.args.binds.clone()
        };
        let priority = task.args.priority;

        let section = util::link_section("text", core);
        const_app.push(quote!(
            #[allow(non_snake_case)]
            #[no_mangle]
            #section
            #cfg_core
            unsafe fn #symbol() {
                const PRIORITY: u8 = #priority;

                #let_instant

                rtfm::export::run(PRIORITY, || {
                    crate::#name(
                        #locals_new
                        #name::Context::new(&rtfm::export::Priority::new(PRIORITY) #instant)
                    )
                });
            }
        ));

        let mut needs_lt = false;

        // `${task}Resources`
        if !task.args.resources.is_empty() {
            let (item, constructor) = resources_struct::codegen(
                Context::HardwareTask(name),
                priority,
                &mut needs_lt,
                app,
                analysis,
            );

            root.push(item);

            const_app.push(constructor);
        }

        root.push(module::codegen(
            Context::HardwareTask(name),
            needs_lt,
            app,
            extra,
        ));

        // `${task}Locals`
        let mut locals_pat = None;
        if !task.locals.is_empty() {
            let (struct_, pat) =
                locals::codegen(Context::HardwareTask(name), &task.locals, core, app);

            root.push(struct_);
            locals_pat = Some(pat);
        }

        let attrs = &task.attrs;
        let context = &task.context;
        let stmts = &task.stmts;
        let section = util::link_section("text", core);
        // XXX shouldn't this have a cfg_core?
        let locals_pat = locals_pat.iter();
        user_tasks.push(quote!(
            #(#attrs)*
            #[allow(non_snake_case)]
            #section
            fn #name(#(#locals_pat,)* #context: #name::Context) {
                use rtfm::Mutex as _;

                #(#stmts)*
            }
        ));
    }

    (const_app, root, user_tasks)
}
