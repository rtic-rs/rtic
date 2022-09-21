use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis, _extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let interrupts = &analysis.interrupts_normal;

    for (&level, channel) in &analysis.channels {
        if channel
            .tasks
            .iter()
            .map(|task_name| app.software_tasks[task_name].is_async)
            .all(|is_async| is_async)
        {
            // check if all tasks are async, if so don't generate this.
            continue;
        }

        let mut stmts = vec![];

        let variants = channel
            .tasks
            .iter()
            .filter(|name| !app.software_tasks[*name].is_async)
            .map(|name| {
                let cfgs = &app.software_tasks[name].cfgs;

                quote!(
                    #(#cfgs)*
                    #name
                )
            })
            .collect::<Vec<_>>();

        // For future use
        // let doc = format!(
        //     "Software tasks to be dispatched at priority level {}",
        //     level,
        // );
        let t = util::spawn_t_ident(level);
        items.push(quote!(
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            // #[doc = #doc]
            #[doc(hidden)]
            pub enum #t {
                #(#variants,)*
            }
        ));

        let n = util::capacity_literal(channel.capacity as usize + 1);
        let rq = util::rq_ident(level);
        // let (_, _, _, input_ty) = util::regroup_inputs(inputs);
        let (rq_ty, rq_expr) = {
            (
                quote!(rtic::export::SCRQ<#t, #n>),
                quote!(rtic::export::Queue::new()),
            )
        };

        // For future use
        // let doc = format!(
        //     "Queue of tasks ready to be dispatched at priority level {}",
        //     level
        // );
        items.push(quote!(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            static #rq: rtic::RacyCell<#rq_ty> = rtic::RacyCell::new(#rq_expr);
        ));

        let interrupt = util::suffixed(
            &interrupts
                .get(&level)
                .expect("RTIC-ICE: Unable to get interrrupt")
                .0
                .to_string(),
        );
        let arms = channel
            .tasks
            .iter()
            .map(|name| {
                let task = &app.software_tasks[name];
                let cfgs = &task.cfgs;
                let fq = util::fq_ident(name);
                let inputs = util::inputs_ident(name);
                let (_, tupled, pats, _) = util::regroup_inputs(&task.inputs);

                if !task.is_async {
                    quote!(
                        #(#cfgs)*
                        #t::#name => {
                            let #tupled =
                                (&*#inputs
                                .get())
                                .get_unchecked(usize::from(index))
                                .as_ptr()
                                .read();
                            (&mut *#fq.get_mut()).split().0.enqueue_unchecked(index);
                            let priority = &rtic::export::Priority::new(PRIORITY);
                            #name(
                                #name::Context::new(priority)
                                #(,#pats)*
                            )
                        }
                    )
                } else {
                    quote!()
                }
            })
            .collect::<Vec<_>>();

        stmts.push(quote!(
            while let Some((task, index)) = (&mut *#rq.get_mut()).split().1.dequeue() {
                match task {
                    #(#arms)*
                }
            }
        ));

        let doc = format!("Interrupt handler to dispatch tasks at priority {}", level);
        let attribute = &interrupts[&level].1.attrs;
        items.push(quote!(
            #[allow(non_snake_case)]
            #[doc = #doc]
            #[no_mangle]
            #(#attribute)*
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
