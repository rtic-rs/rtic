use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    for (&receiver, dispatchers) in &analysis.channels {
        let interrupts = &analysis.interrupts[&receiver];

        for (&level, channels) in dispatchers {
            let mut stmts = vec![];

            for (&sender, channel) in channels {
                let cfg_sender = util::cfg_core(sender, app.args.cores);

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
                    "Software tasks spawned from core #{} to be dispatched at priority level {} by core #{}",
                    sender, level, receiver,
                );
                let t = util::spawn_t_ident(receiver, level, sender);
                items.push(quote!(
                    #[allow(non_camel_case_types)]
                    #[derive(Clone, Copy)]
                    #[doc = #doc]
                    enum #t {
                        #(#variants,)*
                    }
                ));

                let n = util::capacity_typenum(channel.capacity, true);
                let rq = util::rq_ident(receiver, level, sender);
                let (rq_attr, rq_ty, rq_expr, section) = if sender == receiver {
                    (
                        cfg_sender.clone(),
                        quote!(rtfm::export::SCRQ<#t, #n>),
                        quote!(rtfm::export::Queue(unsafe {
                            rtfm::export::iQueue::u8_sc()
                        })),
                        util::link_section("bss", sender),
                    )
                } else {
                    let shared = if cfg!(feature = "heterogeneous") {
                        Some(quote!(#[rtfm::export::shared]))
                    } else {
                        None
                    };

                    (
                        shared,
                        quote!(rtfm::export::MCRQ<#t, #n>),
                        quote!(rtfm::export::Queue(rtfm::export::iQueue::u8())),
                        None,
                    )
                };

                let doc = format!(
                    "Queue of tasks sent by core #{} ready to be dispatched by core #{} at priority level {}",
                    sender,
                    receiver,
                    level
                );
                items.push(quote!(
                    #[doc = #doc]
                    #rq_attr
                    #section
                    static mut #rq: #rq_ty = #rq_expr;
                ));

                if let Some(ceiling) = channel.ceiling {
                    items.push(quote!(
                        #cfg_sender
                        struct #rq<'a> {
                            priority: &'a rtfm::export::Priority,
                        }
                    ));

                    items.push(util::impl_mutex(
                        extra,
                        &[],
                        cfg_sender.as_ref(),
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
                        let fq = util::fq_ident(name, sender);
                        let inputs = util::inputs_ident(name, sender);
                        let (_, tupled, pats, _) = util::regroup_inputs(&task.inputs);

                        let (let_instant, instant) = if app.uses_schedule(receiver) {
                            let instants = util::instants_ident(name, sender);

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
                                let priority = &rtfm::export::Priority::new(PRIORITY);
                                #name(
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
            }

            let doc = format!(
                "Interrupt handler used by core #{} to dispatch tasks at priority {}",
                receiver, level
            );
            let cfg_receiver = util::cfg_core(receiver, app.args.cores);
            let section = util::link_section("text", receiver);
            let interrupt = util::suffixed(&interrupts[&level].to_string(), receiver);
            items.push(quote!(
                #[allow(non_snake_case)]
                #[doc = #doc]
                #[no_mangle]
                #cfg_receiver
                #section
                unsafe fn #interrupt() {
                    /// The priority of this interrupt handler
                    const PRIORITY: u8 = #level;

                    rtfm::export::run(PRIORITY, || {
                        #(#stmts)*
                    });
                }
            ));
        }
    }

    items
}
