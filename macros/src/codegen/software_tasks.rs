use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{locals, module, resources_struct, util},
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
        let cap_lit = util::capacity_literal(cap);
        let cap_ty = util::capacity_typenum(cap, true);

        // Create free queues and inputs / instants buffers
        let fq = util::fq_ident(name);

        let (fq_ty, fq_expr, mk_uninit): (_, _, Box<dyn Fn() -> Option<_>>) = {
            (
                quote!(rtic::export::SCFQ<#cap_ty>),
                quote!(rtic::export::Queue(unsafe {
                    rtic::export::iQueue::u8_sc()
                })),
                Box::new(|| util::link_section_uninit(true)),
            )
        };
        mod_app.push(quote!(
            /// Queue version of a free-list that keeps track of empty slots in
            /// the following buffers
            pub static mut #fq: #fq_ty = #fq_expr;
        ));

        let ref elems = (0..cap)
            .map(|_| quote!(core::mem::MaybeUninit::uninit()))
            .collect::<Vec<_>>();

        if let Some(m) = extra.monotonic {
            let instants = util::instants_ident(name);

            let uninit = mk_uninit();
            mod_app.push(quote!(
                #uninit
                /// Buffer that holds the instants associated to the inputs of a task
                pub static mut #instants:
                    [core::mem::MaybeUninit<<#m as rtic::Monotonic>::Instant>; #cap_lit] =
                    [#(#elems,)*];
            ));
        }

        let uninit = mk_uninit();
        let inputs_ident = util::inputs_ident(name);
        mod_app.push(quote!(
            #uninit
            /// Buffer that holds the inputs of a task
            pub static mut #inputs_ident: [core::mem::MaybeUninit<#input_ty>; #cap_lit] =
                [#(#elems,)*];
        ));

        // `${task}Resources`
        let mut needs_lt = false;
        if !task.args.resources.is_empty() {
            let (item, constructor) = resources_struct::codegen(
                Context::SoftwareTask(name),
                task.args.priority,
                &mut needs_lt,
                app,
                analysis,
            );

            root.push(item);

            mod_app.push(constructor);
        }

        // `${task}Locals`
        let mut locals_pat = None;
        if !task.locals.is_empty() {
            let (struct_, pat) = locals::codegen(Context::SoftwareTask(name), &task.locals, app);

            locals_pat = Some(pat);
            root.push(struct_);
        }

        let context = &task.context;
        let attrs = &task.attrs;
        let cfgs = &task.cfgs;
        let stmts = &task.stmts;
        let locals_pat = locals_pat.iter();
        user_tasks.push(quote!(
            #(#attrs)*
            #(#cfgs)*
            #[allow(non_snake_case)]
            pub fn #name(#(#locals_pat,)* #context: #name::Context #(,#inputs)*) {
                use rtic::Mutex as _;

                #(#stmts)*
            }
        ));

        root.push(module::codegen(
            Context::SoftwareTask(name),
            needs_lt,
            app,
            analysis,
            extra,
        ));
    }

    (mod_app, root, user_tasks)
}
