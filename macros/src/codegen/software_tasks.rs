use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::{ast::App, Context};

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
    // const_app_software_tasks -- free queues, buffers and `${task}Resources` constructors
    Vec<TokenStream2>,
    // root_software_tasks -- items that must be placed in the root of the crate:
    // - `${task}Locals` structs
    // - `${task}Resources` structs
    // - `${task}` modules
    Vec<TokenStream2>,
    // user_software_tasks -- the `#[task]` functions written by the user
    Vec<TokenStream2>,
) {
    let mut const_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];

    for (name, task) in &app.software_tasks {
        let receiver = task.args.core;

        let inputs = &task.inputs;
        let (_, _, _, input_ty) = util::regroup_inputs(inputs);

        let cap = task.args.capacity;
        let cap_lit = util::capacity_literal(cap);
        let cap_ty = util::capacity_typenum(cap, true);

        // create free queues and inputs / instants buffers
        if let Some(free_queues) = analysis.free_queues.get(name) {
            for (&sender, &ceiling) in free_queues {
                let cfg_sender = util::cfg_core(sender, app.args.cores);
                let fq = util::fq_ident(name, sender);

                let (loc, fq_ty, fq_expr, bss, mk_uninit): (
                    _,
                    _,
                    _,
                    _,
                    Box<dyn Fn() -> Option<_>>,
                ) = if receiver == sender {
                    (
                        cfg_sender.clone(),
                        quote!(rtfm::export::SCFQ<#cap_ty>),
                        quote!(rtfm::export::Queue(unsafe {
                            rtfm::export::iQueue::u8_sc()
                        })),
                        util::link_section("bss", sender),
                        Box::new(|| util::link_section_uninit(Some(sender))),
                    )
                } else {
                    let shared = if cfg!(feature = "heterogeneous") {
                        Some(quote!(#[rtfm::export::shared]))
                    } else {
                        None
                    };

                    (
                        shared,
                        quote!(rtfm::export::MCFQ<#cap_ty>),
                        quote!(rtfm::export::Queue(rtfm::export::iQueue::u8())),
                        None,
                        Box::new(|| util::link_section_uninit(None)),
                    )
                };
                let loc = &loc;

                const_app.push(quote!(
                    /// Queue version of a free-list that keeps track of empty slots in
                    /// the following buffers
                    #loc
                    #bss
                    static mut #fq: #fq_ty = #fq_expr;
                ));

                // Generate a resource proxy if needed
                if let Some(ceiling) = ceiling {
                    const_app.push(quote!(
                        #cfg_sender
                        struct #fq<'a> {
                            priority: &'a rtfm::export::Priority,
                        }
                    ));

                    const_app.push(util::impl_mutex(
                        extra,
                        &[],
                        cfg_sender.as_ref(),
                        false,
                        &fq,
                        fq_ty,
                        ceiling,
                        quote!(&mut #fq),
                    ));
                }

                let ref elems = (0..cap)
                    .map(|_| quote!(core::mem::MaybeUninit::uninit()))
                    .collect::<Vec<_>>();

                if app.uses_schedule(receiver) {
                    let m = extra.monotonic();
                    let instants = util::instants_ident(name, sender);

                    let uninit = mk_uninit();
                    const_app.push(quote!(
                        #loc
                        #uninit
                        /// Buffer that holds the instants associated to the inputs of a task
                        static mut #instants:
                            [core::mem::MaybeUninit<<#m as rtfm::Monotonic>::Instant>; #cap_lit] =
                            [#(#elems,)*];
                    ));
                }

                let uninit = mk_uninit();
                let inputs = util::inputs_ident(name, sender);
                const_app.push(quote!(
                    #loc
                    #uninit
                    /// Buffer that holds the inputs of a task
                    static mut #inputs: [core::mem::MaybeUninit<#input_ty>; #cap_lit] =
                        [#(#elems,)*];
                ));
            }
        }

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

            const_app.push(constructor);
        }

        // `${task}Locals`
        let mut locals_pat = None;
        if !task.locals.is_empty() {
            let (struct_, pat) =
                locals::codegen(Context::SoftwareTask(name), &task.locals, receiver, app);

            locals_pat = Some(pat);
            root.push(struct_);
        }

        let cfg_receiver = util::cfg_core(receiver, app.args.cores);
        let section = util::link_section("text", receiver);
        let context = &task.context;
        let attrs = &task.attrs;
        let cfgs = &task.cfgs;
        let stmts = &task.stmts;
        let locals_pat = locals_pat.iter();
        user_tasks.push(quote!(
            #(#attrs)*
            #(#cfgs)*
            #[allow(non_snake_case)]
            #cfg_receiver
            #section
            fn #name(#(#locals_pat,)* #context: #name::Context #(,#inputs)*) {
                use rtfm::Mutex as _;

                #(#stmts)*
            }
        ));

        root.push(module::codegen(
            Context::SoftwareTask(name),
            needs_lt,
            app,
            extra,
        ));
    }

    (const_app, root, user_tasks)
}
