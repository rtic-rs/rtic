use indexmap::IndexMap;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> TokenStream2 {
    // Generate a `Poster` type, and `Post` implementations for every message that a task has
    // subscribed to.

    let mut map: IndexMap<_, Vec<_>> = IndexMap::new();
    for (name, obj) in &app.actors {
        for (subscription_index, subscription) in obj.subscriptions.iter().enumerate() {
            map.entry(subscription.ty.clone())
                .or_default()
                .push((name, subscription_index));
        }
    }

    let post_impls = map.iter().map(|(message_ty, pairs)| {
        let last_index = pairs.len() - 1;
        let any_is_full = pairs
            .iter()
            .map(|(actor_name, subscription_index)| {
                let post_name = util::actor_post(actor_name, *subscription_index);

                quote!(#post_name::is_full())
            })
            .collect::<Vec<_>>();
        let posts = pairs
            .iter()
            .enumerate()
            .map(|(i, (actor_name, subscription_index))| {
                let post_name = util::actor_post(actor_name, *subscription_index);

                if i == last_index {
                    // avoid Clone on last message
                    quote!(#post_name(message)?;)
                } else {
                    quote!(#post_name(message.clone())?;)
                }
            })
            .collect::<Vec<_>>();

        quote! {
            impl rtic::export::Post<#message_ty> for Poster {
                fn post(&mut self, message: #message_ty) -> Result<(), #message_ty> {
                    // TODO(micro-optimization) do the `clone`-ing *outside* the critical section
                    // Atomically posts all messages
                    rtic::export::interrupt::free(|_| unsafe {
                        if false #(|| #any_is_full)* {
                            return Err(message)
                        }
                        #(#posts)*
                        Ok(())
                    })?;
                    Ok(())
                }
            }
        }
    });

    // Actor receive "task" functions
    let mut task_functions = vec![];
    for (name, actor) in &app.actors {
        let actor_ty = &actor.ty;
        for (subscription_index, subscription) in actor.subscriptions.iter().enumerate() {
            let function_name = &util::internal_actor_receive_task(name, subscription_index);
            let actor_state = util::actor_state_ident(name);
            let input_ty = &subscription.ty;
            let refmut = if actor.init.is_none() {
                quote!(&mut *(&mut *#actor_state.get_mut()).as_mut_ptr())
            } else {
                quote!((&mut *#actor_state.get_mut()))
            };

            task_functions.push(quote!(
                fn #function_name(message: #input_ty) {
                    // NOTE(safety) all the Receive methods of an actor instance run at the same
                    // priority so no lock required
                    unsafe {
                        <#actor_ty as rtic::export::Receive<#input_ty>>::receive(
                            #refmut,
                            message,
                        )
                    }
                }
            ));
        }
    }

    // "Spawn" infrastructure
    let mut spawn_infra = vec![];
    for (actor_name, actor) in &app.actors {
        for (subscription_index, subscription) in actor.subscriptions.iter().enumerate() {
            let capacity = subscription.capacity;
            let message_ty = &subscription.ty;

            let cap_lit = util::capacity_literal(capacity as usize);
            let cap_lit_p1 = util::capacity_literal(capacity as usize + 1);
            let pseudo_task_name = util::actor_receive_task(actor_name, subscription_index);
            let inputs_ident = util::inputs_ident(&pseudo_task_name);
            let elems = (0..capacity).map(|_| quote!(core::mem::MaybeUninit::uninit()));

            let uninit_section = util::link_section_uninit();
            spawn_infra.push(quote!(
                #uninit_section
                // /// Buffer that holds the inputs of a task
                #[doc(hidden)]
                static #inputs_ident: rtic::RacyCell<[core::mem::MaybeUninit<#message_ty>; #cap_lit]> =
                    rtic::RacyCell::new([#(#elems,)*]);
            ));

            let fq_ident = util::fq_ident(&pseudo_task_name);

            let fq_ty = quote!(rtic::export::SCFQ<#cap_lit_p1>);
            let fq_expr = quote!(rtic::export::Queue::new());

            spawn_infra.push(quote!(
                // /// Queue version of a free-list that keeps track of empty slots in
                // /// the following buffers
                #[doc(hidden)]
                static #fq_ident: rtic::RacyCell<#fq_ty> = rtic::RacyCell::new(#fq_expr);
            ));

            let priority = actor.priority;
            let t = util::spawn_t_ident(priority);
            let device = &extra.device;
            let enum_ = util::interrupt_ident();
            let interrupt = &analysis
                .interrupts
                .get(&priority)
                .expect("RTIC-ICE: interrupt identifer not found")
                .0;

            let call_update_watermark = if cfg!(feature = "memory-watermark") {
                let update_watermark = util::update_watermark(subscription_index);
                quote!(
                    #actor_name::#update_watermark((*#fq_ident.get()).len());
                )
            } else {
                quote!()
            };

            let rq = util::rq_ident(priority);
            let dequeue = quote!((&mut *#fq_ident.get_mut()).dequeue());
            let post_name = util::actor_post(actor_name, subscription_index);
            spawn_infra.push(quote!(
                /// Safety: needs to be wrapped in a critical section
                unsafe fn #post_name(message: #message_ty) -> Result<(), #message_ty> {
                    unsafe {
                        if let Some(index) = #dequeue {
                            #call_update_watermark
                            (*#inputs_ident
                                .get_mut())
                                    .get_unchecked_mut(usize::from(index))
                                    .as_mut_ptr()
                                    .write(message);

                             (*#rq.get_mut()).enqueue_unchecked((#t::#pseudo_task_name, index));

                            rtic::pend(#device::#enum_::#interrupt);

                            Ok(())
                        } else {
                            Err(message)
                        }
                    }
                }

                mod #post_name {
                    /// Safety: needs to be wrapped in a critical section
                    pub unsafe fn is_full() -> bool {
                        // this is the queue version of a "free list" when it's empty the message
                        // queue of the task is full (= no more messages can be posted)
                        (&*super::#fq_ident.get()).len() == 0
                    }
                }
            ));
        }
    }

    // watermark API
    let watermark_api = if cfg!(feature = "memory-watermark") {
        watermark_api(app)
    } else {
        quote!()
    };

    quote! {
        // Make `Post` methods available in the app module.
        use rtic::export::Post as _;

        #[derive(Clone, Copy)]
        pub struct Poster;

        #(#post_impls)*

        #(#task_functions)*

        #(#spawn_infra)*

        #watermark_api
    }
}

fn watermark_api(app: &App) -> TokenStream2 {
    let mut actor_mods = vec![];
    for (actor_name, actor) in &app.actors {
        if actor.subscriptions.is_empty() {
            // skip disconnected actors
            continue;
        }

        let mut mod_items = vec![];
        let mut subscriptions_elements = vec![];
        for (subscription_index, subscription) in actor.subscriptions.iter().enumerate() {
            let capacity = util::capacity_literal(subscription.capacity.into());

            let counter = format_ident!("COUNTER{}", subscription_index);
            let update_watermark = util::update_watermark(subscription_index);
            let watermark = util::watermark(subscription_index);
            mod_items.push(quote!(
                static #counter: AtomicUsize = AtomicUsize::new(0);

                pub fn #update_watermark(fq_len: usize) {
                    let new_usage = #capacity - fq_len;
                    if new_usage > #counter.load(Ordering::Relaxed) {
                        #counter.store(new_usage, Ordering::Relaxed)
                    }
                }

                pub fn #watermark() -> usize {
                    #counter.load(Ordering::Relaxed)
                }
            ));

            let ty = &subscription.ty;
            let message_type = quote!(#ty).to_string();
            subscriptions_elements.push(quote!(
                Subscription {
                    capacity: #capacity,
                    message_type: #message_type,
                    watermark: #watermark,
                }
            ));
        }

        actor_mods.push(quote!(
            pub mod #actor_name {
                use core::sync::atomic::{AtomicUsize, Ordering};

                use super::Subscription;

                pub static SUBSCRIPTIONS: &[Subscription] =
                    &[#(#subscriptions_elements),*];

                #(#mod_items)*
            }
        ))
    }

    if actor_mods.is_empty() {
        // all actors are disconnected
        return quote!();
    }

    // NOTE this API could live in a crate like `rtic-core`
    let subscription_api = quote!(
        pub struct Subscription {
            pub capacity: usize,
            pub message_type: &'static str,
            watermark: fn() -> usize,
        }

        impl Subscription {
            pub fn watermark(&self) -> usize {
                (self.watermark)()
            }
        }

        impl core::fmt::Debug for Subscription {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Subscription")
                    .field("capacity", &self.capacity)
                    .field("message_type", &self.message_type)
                    .field("watermark", &self.watermark())
                    .finish()
            }
        }
    );

    quote!(
        #subscription_api

        #(#actor_mods)*
    )
}
