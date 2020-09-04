use std::collections::HashSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{spawn_body, util},
};

/// Generates all `${ctxt}::Spawn` methods
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let mut seen = HashSet::<_>::new();
    for (spawner, spawnees) in app.spawn_callers() {
        let mut methods = vec![];

        for name in spawnees {
            let spawnee = &app.software_tasks[name];
            let cfgs = &spawnee.cfgs;
            let (args, _, untupled, ty) = util::regroup_inputs(&spawnee.inputs);
            let args = &args;

            if spawner.is_init() {
                // `init` uses a special spawn implementation; it doesn't use the `spawn_${name}`
                // functions which are shared by other contexts

                let body = spawn_body::codegen(spawner, &name, app, analysis, extra);

                let let_instant = if app.uses_schedule() {
                    let m = extra.monotonic();

                    Some(quote!(let instant = unsafe { <#m as rtic::Monotonic>::zero() };))
                } else {
                    None
                };

                methods.push(quote!(
                    #(#cfgs)*
                    fn #name(&self #(,#args)*) -> Result<(), #ty> {
                        #let_instant
                        #body
                    }
                ));
            } else {
                let spawn = util::spawn_ident(name);

                if !seen.contains(name) {
                    // Generate a `spawn_${name}_S${sender}` function
                    seen.insert(name);

                    let instant = if app.uses_schedule() {
                        let m = extra.monotonic();

                        Some(quote!(, instant: <#m as rtic::Monotonic>::Instant))
                    } else {
                        None
                    };

                    let body = spawn_body::codegen(spawner, &name, app, analysis, extra);

                    items.push(quote!(
                        #(#cfgs)*
                        unsafe fn #spawn(
                            priority: &rtic::export::Priority
                            #instant
                            #(,#args)*
                        ) -> Result<(), #ty> {
                            #body
                        }
                    ));
                }

                let (let_instant, instant) = if app.uses_schedule() {
                    let m = extra.monotonic();

                    (
                        Some(if spawner.is_idle() {
                            quote!(let instant = <#m as rtic::Monotonic>::now();)
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
            impl<#lt> #spawner::Spawn<#lt> {
                #(#methods)*
            }
        ));
    }

    items
}
