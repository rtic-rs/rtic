use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis, _extra: &Extra) -> Vec<TokenStream2> {
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
            pub enum #t {
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

        let arms = channel
            .tasks
            .iter()
            .map(|name| {
                let task = &app.software_tasks[name];
                let cfgs = &task.cfgs;
                let fq = util::fq_ident(name);
                let inputs = util::inputs_ident(name);
                let (_, tupled, pats, _) = util::regroup_inputs(&task.inputs);

                let locals_new = if task.locals.is_empty() {
                    quote!()
                } else {
                    quote!(#name::Locals::new(),)
                };

                let app_name = &app.name;
                let app_path = quote! {crate::#app_name};
                quote!(
                    #(#cfgs)*
                    #t::#name => {
                        let #tupled =
                            #inputs.get_unchecked(usize::from(index)).as_ptr().read();
                        #fq.split().0.enqueue_unchecked(index);
                        let priority = &rtic::export::Priority::new(PRIORITY);
                        #app_path::#name(
                            #locals_new
                            #name::Context::new(priority)
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
        let interrupt = util::suffixed(&interrupts[&level].0.to_string());
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
