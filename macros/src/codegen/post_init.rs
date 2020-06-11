use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates code that runs after `#[init]` returns
pub fn codegen(
    core: u8,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    let mut const_app = vec![];
    let mut stmts = vec![];

    // initialize late resources
    if let Some(late_resources) = analysis.late_resources.get(&core) {
        for name in late_resources {
            // if it's live
            let cfgs = app.late_resources[name].cfgs.clone();
            if analysis.locations.get(name).is_some() {
                // Need to also include the cfgs
                stmts.push(quote!(
                #(#cfgs)*
                #name.as_mut_ptr().write(late.#name);
                ));
            }
        }
    }

    if analysis.timer_queues.is_empty() {
        // cross-initialization barriers -- notify *other* cores that their resources have been
        // initialized
        for (user, initializers) in &analysis.initialization_barriers {
            if !initializers.contains(&core) {
                continue;
            }

            let ib = util::init_barrier(*user);
            let shared = if cfg!(feature = "heterogeneous") {
                Some(quote!(
                    #[rtic::export::shared]
                ))
            } else {
                None
            };

            const_app.push(quote!(
                #shared
                static #ib: rtic::export::Barrier = rtic::export::Barrier::new();
            ));

            stmts.push(quote!(
                #ib.release();
            ));
        }

        // then wait until the other cores have initialized *our* resources
        if analysis.initialization_barriers.contains_key(&core) {
            let ib = util::init_barrier(core);

            stmts.push(quote!(
                #ib.wait();
            ));
        }

        // cross-spawn barriers: wait until other cores are ready to receive messages
        for (&receiver, senders) in &analysis.spawn_barriers {
            if senders.get(&core) == Some(&false) {
                let sb = util::spawn_barrier(receiver);

                stmts.push(quote!(
                    #sb.wait();
                ));
            }
        }
    } else {
        // if the `schedule` API is used then we'll synchronize all cores to leave the
        // `init`-ialization phase at the same time. In this case the rendezvous barrier makes the
        // cross-initialization and spawn barriers unnecessary

        let m = extra.monotonic();

        if analysis.timer_queues.len() == 1 {
            // reset the monotonic timer / counter
            stmts.push(quote!(
                <#m as rtic::Monotonic>::reset();
            ));
        } else {
            // in the multi-core case we need a rendezvous (RV) barrier between *all* the cores that
            // use the `schedule` API; otherwise one of the cores could observe the before-reset
            // value of the monotonic counter
            // (this may be easier to implement with `AtomicU8.fetch_sub` but that API is not
            // available on ARMv6-M)

            // this core will reset the monotonic counter
            const FIRST: u8 = 0;

            if core == FIRST {
                for &i in analysis.timer_queues.keys() {
                    let rv = util::rendezvous_ident(i);
                    let shared = if cfg!(feature = "heterogeneous") {
                        Some(quote!(
                            #[rtic::export::shared]
                        ))
                    } else {
                        None
                    };

                    const_app.push(quote!(
                        #shared
                        static #rv: rtic::export::Barrier = rtic::export::Barrier::new();
                    ));

                    // wait until all the other cores have reached the RV point
                    if i != FIRST {
                        stmts.push(quote!(
                            #rv.wait();
                        ));
                    }
                }

                let rv = util::rendezvous_ident(core);
                stmts.push(quote!(
                    // the compiler fences are used to prevent `reset` from being re-ordering wrt to
                    // the atomic operations -- we don't know if `reset` contains load or store
                    // operations

                    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

                    // reset the counter
                    <#m as rtic::Monotonic>::reset();

                    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

                    // now unblock all the other cores
                    #rv.release();
                ));
            } else {
                let rv = util::rendezvous_ident(core);

                // let the first core know that we have reached the RV point
                stmts.push(quote!(
                    #rv.release();
                ));

                let rv = util::rendezvous_ident(FIRST);

                // wait until the first core has reset the monotonic timer
                stmts.push(quote!(
                    #rv.wait();
                ));
            }
        }
    }

    // enable the interrupts -- this completes the `init`-ialization phase
    stmts.push(quote!(rtic::export::interrupt::enable();));

    (const_app, stmts)
}
