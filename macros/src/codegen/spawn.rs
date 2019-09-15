use std::collections::{BTreeMap, HashSet};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::ast::App;

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{spawn_body, util},
};

/// Generates all `${ctxt}::Spawn` methods
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let mut seen = BTreeMap::<u8, HashSet<_>>::new();
    for (spawner, spawnees) in app.spawn_callers() {
        let sender = spawner.core(app);
        let cfg_sender = util::cfg_core(sender, app.args.cores);
        let seen = seen.entry(sender).or_default();
        let mut methods = vec![];

        for name in spawnees {
            let spawnee = &app.software_tasks[name];
            let receiver = spawnee.args.core;
            let cfgs = &spawnee.cfgs;
            let (args, _, untupled, ty) = util::regroup_inputs(&spawnee.inputs);
            let args = &args;

            if spawner.is_init() {
                // `init` uses a special spawn implementation; it doesn't use the `spawn_${name}`
                // functions which are shared by other contexts

                let body = spawn_body::codegen(spawner, &name, app, analysis, extra);

                let let_instant = if app.uses_schedule(receiver) {
                    let m = extra.monotonic();

                    Some(quote!(let instant = unsafe { <#m as rtfm::Monotonic>::zero() };))
                } else {
                    None
                };

                let section = util::link_section("text", sender);
                methods.push(quote!(
                    #(#cfgs)*
                    #section
                    fn #name(&self #(,#args)*) -> Result<(), #ty> {
                        #let_instant
                        #body
                    }
                ));
            } else {
                let spawn = util::spawn_ident(name, sender);

                if !seen.contains(name) {
                    // generate a `spawn_${name}_S${sender}` function
                    seen.insert(name);

                    let instant = if app.uses_schedule(receiver) {
                        let m = extra.monotonic();

                        Some(quote!(, instant: <#m as rtfm::Monotonic>::Instant))
                    } else {
                        None
                    };

                    let body = spawn_body::codegen(spawner, &name, app, analysis, extra);

                    let section = util::link_section("text", sender);
                    items.push(quote!(
                        #cfg_sender
                        #(#cfgs)*
                        #section
                        unsafe fn #spawn(
                            priority: &rtfm::export::Priority
                            #instant
                            #(,#args)*
                        ) -> Result<(), #ty> {
                            #body
                        }
                    ));
                }

                let (let_instant, instant) = if app.uses_schedule(receiver) {
                    let m = extra.monotonic();

                    (
                        Some(if spawner.is_idle() {
                            quote!(let instant = <#m as rtfm::Monotonic>::now();)
                        } else {
                            quote!(let instant = self.instant();)
                        }),
                        Some(quote!(, instant)),
                    )
                } else {
                    (None, None)
                };

                methods.push(quote!(
                    #(#cfgs)*
                    #[inline(always)]
                    fn #name(&self #(,#args)*) -> Result<(), #ty> {
                        unsafe {
                            #let_instant
                            #spawn(self.priority() #instant #(,#untupled)*)
                        }
                    }
                ));
            }
        }

        let lt = if spawner.is_init() {
            None
        } else {
            Some(quote!('a))
        };

        let spawner = spawner.ident(app);
        debug_assert!(!methods.is_empty());
        items.push(quote!(
            #cfg_sender
            impl<#lt> #spawner::Spawn<#lt> {
                #(#methods)*
            }
        ));
    }

    items
}
