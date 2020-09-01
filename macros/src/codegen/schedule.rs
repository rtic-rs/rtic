use std::collections::HashSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{
    check::Extra,
    codegen::{schedule_body, util},
};

/// Generates all `${ctxt}::Schedule` methods
pub fn codegen(app: &App, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let mut seen = HashSet::<_>::new();
    for (scheduler, schedulees) in app.schedule_callers() {
        let m = extra.monotonic();
        let instant = quote!(<#m as rtic::Monotonic>::Instant);

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

                methods.push(quote!(
                    #(#cfgs)*
                    fn #name(&self, instant: #instant #(,#args)*) -> Result<(), #ty> {
                        #body
                    }
                ));
            } else {
                let schedule = util::schedule_ident(name);

                if !seen.contains(name) {
                    // Generate a `schedule_${name}_S${sender}` function
                    seen.insert(name);

                    let body = schedule_body::codegen(scheduler, &name, app);

                    items.push(quote!(
                        #(#cfgs)*
                        unsafe fn #schedule(
                            priority: &rtic::export::Priority,
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
            impl<#lt> #scheduler::Schedule<#lt> {
                #(#methods)*
            }
        ));
    }

    items
}
