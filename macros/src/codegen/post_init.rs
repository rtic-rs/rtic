use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{locals, util},
};

/// Generates code that runs after `#[init]` returns
pub fn codegen(
    core: u8,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // const_app
    Vec<TokenStream2>,
    // root
    Vec<TokenStream2>,
    // stmts
    Vec<TokenStream2>,
) {
    let mut const_app = vec![];
    let mut root = vec![];
    let mut stmts = vec![];

    // initialize late resources
    if let Some(late_resources) = analysis.late_resources.get(&core) {
        for name in late_resources {
            // if it's live
            if analysis.locations.get(name).is_some() {
                stmts.push(quote!(#name.as_mut_ptr().write(late.#name);));
            }
        }
    }

    // TODO WIP
    for (name, task) in &app.hardware_tasks {
        if task.is_generator {
            let name_s = name.to_string();
            let gen_i = util::generator_ident(&name_s);
            let gen_t = util::generator_type(&name_s);
            const_app.push(quote!(
                static mut #gen_i: core::mem::MaybeUninit<#gen_t> =
                    core::mem::MaybeUninit::uninit();
            ));

            let (locals_pat, locals_new) = if task.locals.is_empty() {
                (None, quote!())
            } else {
                let (struct_, pat) =
                    locals::codegen(Context::HardwareTask(name), &task.locals, core, app);

                root.push(struct_);

                (Some(pat), quote!(#name::Locals::new(),))
            };

            let context = &task.context;
            let task_stmts = &task.stmts;
            let locals_pat = locals_pat.iter();
            root.push(quote!(
                type #gen_t = impl core::ops::Generator<Yield = (), Return = !>;

                // #[allow(non_snake_case)]
                fn #name(#(#locals_pat,)* #context: #name::Context) -> #gen_t {
                    use rtfm::Mutex as _;

                    #(#task_stmts)*
                }
            ));

            stmts.push(quote!(
                #gen_i.as_mut_ptr().write(#name(#locals_new #name::Context::new()));
            ));
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
                    #[rtfm::export::shared]
                ))
            } else {
                None
            };

            const_app.push(quote!(
                #shared
                static #ib: rtfm::export::Barrier = rtfm::export::Barrier::new();
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
                <#m as rtfm::Monotonic>::reset();
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
                            #[rtfm::export::shared]
                        ))
                    } else {
                        None
                    };

                    const_app.push(quote!(
                        #shared
                        static #rv: rtfm::export::Barrier = rtfm::export::Barrier::new();
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
                    <#m as rtfm::Monotonic>::reset();

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
    stmts.push(quote!(rtfm::export::interrupt::enable();));

    (const_app, root, stmts)
}
