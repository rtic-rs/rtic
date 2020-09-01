use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};
use syn::Ident;

use crate::{analyze::Analysis, check::Extra, codegen::util};

pub fn codegen(
    spawner: Context,
    name: &Ident,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> TokenStream2 {
    let spawnee = &app.software_tasks[name];
    let priority = spawnee.args.priority;

    let write_instant = if app.uses_schedule() {
        let instants = util::instants_ident(name);

        Some(quote!(
            #instants.get_unchecked_mut(usize::from(index)).as_mut_ptr().write(instant);
        ))
    } else {
        None
    };

    let t = util::spawn_t_ident(priority);
    let fq = util::fq_ident(name);
    let rq = util::rq_ident(priority);
    let (dequeue, enqueue) = if spawner.is_init() {
        (
            quote!(#fq.dequeue()),
            quote!(#rq.enqueue_unchecked((#t::#name, index));),
        )
    } else {
        (
            quote!((#fq { priority }.lock(|fq| fq.split().1.dequeue()))),
            quote!((#rq { priority }.lock(|rq| {
                rq.split().0.enqueue_unchecked((#t::#name, index))
            }));),
        )
    };

    let device = extra.device;
    let enum_ = util::interrupt_ident();
    let interrupt = &analysis.interrupts.get(&priority);
    let pend = {
        quote!(
            rtic::pend(#device::#enum_::#interrupt);
        )
    };

    let (_, tupled, _, _) = util::regroup_inputs(&spawnee.inputs);
    let inputs = util::inputs_ident(name);
    quote!(
        unsafe {
            use rtic::Mutex as _;

            let input = #tupled;
            if let Some(index) = #dequeue {
                #inputs.get_unchecked_mut(usize::from(index)).as_mut_ptr().write(input);

                #write_instant

                #enqueue

                #pend

                Ok(())
            } else {
                Err(input)
            }
        }
    )
}
