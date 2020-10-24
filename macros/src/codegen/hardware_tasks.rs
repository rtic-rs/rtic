use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{locals, module, resources_struct},
};

/// Generate support code for hardware tasks (`#[exception]`s and `#[interrupt]`s)
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // mod_app_hardware_tasks -- interrupt handlers and `${task}Resources` constructors
    Vec<TokenStream2>,
    // root_hardware_tasks -- items that must be placed in the root of the crate:
    // - `${task}Locals` structs
    // - `${task}Resources` structs
    // - `${task}` modules
    Vec<TokenStream2>,
    // user_hardware_tasks -- the `#[task]` functions written by the user
    Vec<TokenStream2>,
) {
    let mut mod_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];

    for (name, task) in &app.hardware_tasks {
        let (let_instant, instant) = if let Some(ref m) = extra.monotonic {
            (
                Some(quote!(let instant = <#m as rtic::Monotonic>::now();)),
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

        let symbol = task.args.binds.clone();
        let priority = task.args.priority;

        let app_name = &app.name;
        let app_path = quote! {crate::#app_name};
        mod_app.push(quote!(
            #[allow(non_snake_case)]
            #[no_mangle]
            unsafe fn #symbol() {
                const PRIORITY: u8 = #priority;

                #let_instant

                rtic::export::run(PRIORITY, || {
                    #app_path::#name(
                        #locals_new
                        #name::Context::new(&rtic::export::Priority::new(PRIORITY) #instant)
                    )
                });
            }
        ));

        let mut needs_lt = false;

        // `${task}Resources`
        if !task.args.resources.is_empty() {
            let (item, constructor) =
                resources_struct::codegen(Context::HardwareTask(name), &mut needs_lt, app);

            root.push(item);

            mod_app.push(constructor);
        }

        root.push(module::codegen(
            Context::HardwareTask(name),
            needs_lt,
            app,
            analysis,
            extra,
        ));

        // `${task}Locals`
        let mut locals_pat = None;
        if !task.locals.is_empty() {
            let (struct_, pat) = locals::codegen(Context::HardwareTask(name), &task.locals, app);

            root.push(struct_);
            locals_pat = Some(pat);
        }

        if !&task.is_extern {
            let attrs = &task.attrs;
            let context = &task.context;
            let stmts = &task.stmts;
            let locals_pat = locals_pat.iter();
            user_tasks.push(quote!(
                #(#attrs)*
                #[allow(non_snake_case)]
                fn #name(#(#locals_pat,)* #context: #name::Context) {
                    use rtic::Mutex as _;

                    #(#stmts)*
                }
            ));
        }
    }

    (mod_app, root, user_tasks)
}
