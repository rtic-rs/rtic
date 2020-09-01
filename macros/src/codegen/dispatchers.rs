use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let interrupts = &analysis.interrupts;

    for (&level, channel) in &analysis.channels {
        let mut stmts = vec![];

        let variants = channel
            .tasks
            .iter()
            .map(|name| {
                let cfgs = &app.software_tasks[name].cfgs;

                quote!(
                    #(#cfgs)*
                    #name
                )
            })
            .collect::<Vec<_>>();

        let doc = format!(
            "Software tasks to be dispatched at priority level {}",
            level,
        );
        let t = util::spawn_t_ident(level);
        items.push(quote!(
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            #[doc = #doc]
            enum #t {
                #(#variants,)*
            }
        ));

        let n = util::capacity_typenum(channel.capacity, true);
        let rq = util::rq_ident(level);
        let (rq_ty, rq_expr) = {
            (
                quote!(rtic::export::SCRQ<#t, #n>),
                quote!(rtic::export::Queue(unsafe {
                    rtic::export::iQueue::u8_sc()
                })),
            )
        };

        let doc = format!(
            "Queue of tasks ready to be dispatched at priority level {}",
            level
        );
        items.push(quote!(
            #[doc = #doc]
            static mut #rq: #rq_ty = #rq_expr;
        ));

        if let Some(ceiling) = channel.ceiling {
            items.push(quote!(
                struct #rq<'a> {
                    priority: &'a rtic::export::Priority,
                }
            ));

            items.push(util::impl_mutex(
                extra,
                &[],
                false,
                &rq,
                rq_ty,
                ceiling,
                quote!(&mut #rq),
            ));
        }

        let arms = channel
            .tasks
            .iter()
            .map(|name| {
                let task = &app.software_tasks[name];
                let cfgs = &task.cfgs;
                let fq = util::fq_ident(name);
                let inputs = util::inputs_ident(name);
                let (_, tupled, pats, _) = util::regroup_inputs(&task.inputs);

                let (let_instant, instant) = if app.uses_schedule() {
                    let instants = util::instants_ident(name);

                    (
                        quote!(
                            let instant =
                                #instants.get_unchecked(usize::from(index)).as_ptr().read();
                        ),
                        quote!(, instant),
                    )
                } else {
                    (quote!(), quote!())
                };

                let locals_new = if task.locals.is_empty() {
                    quote!()
                } else {
                    quote!(#name::Locals::new(),)
                };

                quote!(
                    #(#cfgs)*
                    #t::#name => {
                        let #tupled =
                            #inputs.get_unchecked(usize::from(index)).as_ptr().read();
                        #let_instant
                        #fq.split().0.enqueue_unchecked(index);
                        let priority = &rtic::export::Priority::new(PRIORITY);
                        crate::#name(
                            #locals_new
                            #name::Context::new(priority #instant)
                            #(,#pats)*
                        )
                    }
                )
            })
            .collect::<Vec<_>>();

        stmts.push(quote!(
            while let Some((task, index)) = #rq.split().1.dequeue() {
                match task {
                    #(#arms)*
                }
            }
        ));

        let doc = format!("Interrupt handler to dispatch tasks at priority {}", level);
        let interrupt = util::suffixed(&interrupts[&level].to_string());
        items.push(quote!(
            #[allow(non_snake_case)]
            #[doc = #doc]
            #[no_mangle]
            unsafe fn #interrupt() {
                /// The priority of this interrupt handler
                const PRIORITY: u8 = #level;

                rtic::export::run(PRIORITY, || {
                    #(#stmts)*
                });
            }
        ));
    }

    items
}
