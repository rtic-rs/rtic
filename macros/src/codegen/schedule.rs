use std::collections::{BTreeMap, HashSet};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::ast::App;

use crate::{
    check::Extra,
    codegen::{schedule_body, util},
};

/// Generates all `${ctxt}::Schedule` methods
pub fn codegen(app: &App, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let mut seen = BTreeMap::<u8, HashSet<_>>::new();
    for (scheduler, schedulees) in app.schedule_callers() {
        let m = extra.monotonic();
        let instant = quote!(<#m as rtfm::Monotonic>::Instant);

        let sender = scheduler.core(app);
        let cfg_sender = util::cfg_core(sender, app.args.cores);
        let seen = seen.entry(sender).or_default();
        let mut methods = vec![];

        for name in schedulees {
            let schedulee = &app.software_tasks[name];
            let cfgs = &schedulee.cfgs;
            let (args, _, untupled, ty) = util::regroup_inputs(&schedulee.inputs);
            let args = &args;

            if scheduler.is_init() {
                // `init` uses a special `schedule` implementation; it doesn't use the
                // `schedule_${name}` functions which are shared by other contexts

                let body = schedule_body::codegen(scheduler, &name, app);

                let section = util::link_section("text", sender);
                methods.push(quote!(
                    #(#cfgs)*
                    #section
                    fn #name(&self, instant: #instant #(,#args)*) -> Result<(), #ty> {
                        #body
                    }
                ));
            } else {
                let schedule = util::schedule_ident(name, sender);

                if !seen.contains(name) {
                    // generate a `schedule_${name}_S${sender}` function
                    seen.insert(name);

                    let body = schedule_body::codegen(scheduler, &name, app);

                    let section = util::link_section("text", sender);
                    items.push(quote!(
                        #cfg_sender
                        #(#cfgs)*
                        #section
                        unsafe fn #schedule(
                            priority: &rtfm::export::Priority,
                            instant: #instant
                            #(,#args)*
                        ) -> Result<(), #ty> {
                            #body
                        }
                    ));
                }

                methods.push(quote!(
                    #(#cfgs)*
                    #[inline(always)]
                    fn #name(&self, instant: #instant #(,#args)*) -> Result<(), #ty> {
                        unsafe {
                            #schedule(self.priority(), instant #(,#untupled)*)
                        }
                    }
                ));
            }
        }

        let lt = if scheduler.is_init() {
            None
        } else {
            Some(quote!('a))
        };

        let scheduler = scheduler.ident(app);
        debug_assert!(!methods.is_empty());
        items.push(quote!(
            #cfg_sender
            impl<#lt> #scheduler::Schedule<#lt> {
                #(#methods)*
            }
        ));
    }

    items
}
