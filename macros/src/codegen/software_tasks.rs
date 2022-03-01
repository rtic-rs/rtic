use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{local_resources_struct, module, shared_resources_struct, util},
};

pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // mod_app_software_tasks -- free queues, buffers and `${task}Resources` constructors
    Vec<TokenStream2>,
    // root_software_tasks -- items that must be placed in the root of the crate:
    // - `${task}Locals` structs
    // - `${task}Resources` structs
    // - `${task}` modules
    Vec<TokenStream2>,
    // user_software_tasks -- the `#[task]` functions written by the user
    Vec<TokenStream2>,
) {
    let mut mod_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];

    for (name, task) in &app.software_tasks {
        let inputs = &task.inputs;
        let (_, _, _, input_ty) = util::regroup_inputs(inputs);

        let cap = task.args.capacity;
        let cap_lit = util::capacity_literal(cap as usize);
        let cap_lit_p1 = util::capacity_literal(cap as usize + 1);

        // Create free queues and inputs / instants buffers
        let fq = util::fq_ident(name);

        #[allow(clippy::redundant_closure)]
        let (fq_ty, fq_expr, mk_uninit): (_, _, Box<dyn Fn() -> Option<_>>) = {
            (
                quote!(rtic::export::SCFQ<#cap_lit_p1>),
                quote!(rtic::export::Queue::new()),
                Box::new(|| Some(util::link_section_uninit())),
            )
        };
        mod_app.push(quote!(
            // /// Queue version of a free-list that keeps track of empty slots in
            // /// the following buffers
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            #[doc(hidden)]
            static #fq: rtic::RacyCell<#fq_ty> = rtic::RacyCell::new(#fq_expr);
        ));

        let elems = &(0..cap)
            .map(|_| quote!(core::mem::MaybeUninit::uninit()))
            .collect::<Vec<_>>();

        for (_, monotonic) in &app.monotonics {
            let instants = util::monotonic_instants_ident(name, &monotonic.ident);
            let mono_type = &monotonic.ty;

            let uninit = mk_uninit();
            // For future use
            // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
            mod_app.push(quote!(
                #uninit
                // /// Buffer that holds the instants associated to the inputs of a task
                // #[doc = #doc]
                #[allow(non_camel_case_types)]
                #[allow(non_upper_case_globals)]
                #[doc(hidden)]
                static #instants:
                    rtic::RacyCell<[core::mem::MaybeUninit<<#mono_type as rtic::Monotonic>::Instant>; #cap_lit]> =
                    rtic::RacyCell::new([#(#elems,)*]);
            ));
        }

        let uninit = mk_uninit();
        let inputs_ident = util::inputs_ident(name);
        mod_app.push(quote!(
            #uninit
            // /// Buffer that holds the inputs of a task
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            #[doc(hidden)]
            static #inputs_ident: rtic::RacyCell<[core::mem::MaybeUninit<#input_ty>; #cap_lit]> =
                rtic::RacyCell::new([#(#elems,)*]);
        ));

        // `${task}Resources`
        let mut shared_needs_lt = false;
        let mut local_needs_lt = false;

        // `${task}Locals`
        if !task.args.local_resources.is_empty() {
            let (item, constructor) = local_resources_struct::codegen(
                Context::SoftwareTask(name),
                &mut local_needs_lt,
                app,
            );

            root.push(item);

            mod_app.push(constructor);
        }

        if !task.args.shared_resources.is_empty() {
            let (item, constructor) = shared_resources_struct::codegen(
                Context::SoftwareTask(name),
                &mut shared_needs_lt,
                app,
            );

            root.push(item);

            mod_app.push(constructor);
        }

        if !&task.is_extern {
            let context = &task.context;
            let attrs = &task.attrs;
            let cfgs = &task.cfgs;
            let stmts = &task.stmts;
            user_tasks.push(quote!(
                #(#attrs)*
                #(#cfgs)*
                #[allow(non_snake_case)]
                fn #name(#context: #name::Context #(,#inputs)*) {
                    use rtic::Mutex as _;
                    use rtic::mutex::prelude::*;

                    #(#stmts)*
                }
            ));
        }

        root.push(module::codegen(
            Context::SoftwareTask(name),
            shared_needs_lt,
            local_needs_lt,
            app,
            analysis,
            extra,
        ));
    }

    (mod_app, root, user_tasks)
}
