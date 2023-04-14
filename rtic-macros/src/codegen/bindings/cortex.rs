use crate::{
    analyze::Analysis as CodegenAnalysis,
    codegen::util,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{parse, Attribute, Ident};

#[cfg(feature = "cortex-m-basepri")]
pub use basepri::*;
#[cfg(feature = "cortex-m-source-masking")]
pub use source_masking::*;

/// Whether `name` is an exception with configurable priority
fn is_exception(name: &Ident) -> bool {
    let s = name.to_string();

    matches!(
        &*s,
        "MemoryManagement"
            | "BusFault"
            | "UsageFault"
            | "SecureFault"
            | "SVCall"
            | "DebugMonitor"
            | "PendSV"
            | "SysTick"
    )
}

#[cfg(feature = "cortex-m-source-masking")]
mod source_masking {
    use super::*;
    use std::collections::HashMap;

    /// Generates a `Mutex` implementation
    #[allow(clippy::too_many_arguments)]
    pub fn impl_mutex(
        app: &App,
        analysis: &CodegenAnalysis,
        cfgs: &[Attribute],
        resources_prefix: bool,
        name: &Ident,
        ty: &TokenStream2,
        ceiling: u8,
        ptr: &TokenStream2,
    ) -> TokenStream2 {
        let path = if resources_prefix {
            quote!(shared_resources::#name)
        } else {
            quote!(#name)
        };

        // Computing mapping of used interrupts to masks
        let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

        let mut prio_to_masks = HashMap::new();
        let device = &app.args.device;
        // let mut uses_exceptions_with_resources = false;

        let mut mask_ids = Vec::new();

        for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().flat_map(|task| {
            if !is_exception(&task.args.binds) {
                Some((&task.args.priority, &task.args.binds))
            } else {
                None
            }
        })) {
            let v: &mut Vec<_> = prio_to_masks.entry(priority - 1).or_default();
            v.push(quote!(#device::Interrupt::#name as u32));
            mask_ids.push(quote!(#device::Interrupt::#name as u32));
        }

        // Call rtic::export::create_mask([Mask; N]), where the array is the list of shifts

        let mut mask_arr = Vec::new();
        // NOTE: 0..3 assumes max 4 priority levels according to M0, M23 spec
        for i in 0..3 {
            let v = if let Some(v) = prio_to_masks.get(&i) {
                v.clone()
            } else {
                Vec::new()
            };

            mask_arr.push(quote!(
                rtic::export::create_mask([#(#v),*])
            ));
        }

        quote!(
            #(#cfgs)*
            impl<'a> rtic::Mutex for #path<'a> {
                type T = #ty;

                #[inline(always)]
                fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {
                    /// Priority ceiling
                    const CEILING: u8 = #ceiling;
                    const N_CHUNKS: usize = rtic::export::compute_mask_chunks([#(#mask_ids),*]);
                    const MASKS: [rtic::export::Mask<N_CHUNKS>; 3] = [#(#mask_arr),*];

                    unsafe {
                        rtic::export::lock(
                            #ptr,
                            CEILING,
                            &MASKS,
                            f,
                        )
                    }
                }
            }
        )
    }

    pub fn extra_assertions(_: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![]
    }
}

#[cfg(feature = "cortex-m-basepri")]
mod basepri {
    use super::*;

    /// Generates a `Mutex` implementation
    #[allow(clippy::too_many_arguments)]
    pub fn impl_mutex(
        app: &App,
        _analysis: &CodegenAnalysis,
        cfgs: &[Attribute],
        resources_prefix: bool,
        name: &Ident,
        ty: &TokenStream2,
        ceiling: u8,
        ptr: &TokenStream2,
    ) -> TokenStream2 {
        let path = if resources_prefix {
            quote!(shared_resources::#name)
        } else {
            quote!(#name)
        };

        let device = &app.args.device;
        quote!(
            #(#cfgs)*
            impl<'a> rtic::Mutex for #path<'a> {
                type T = #ty;

                #[inline(always)]
                fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {
                    /// Priority ceiling
                    const CEILING: u8 = #ceiling;

                    unsafe {
                        rtic::export::lock(
                            #ptr,
                            CEILING,
                            #device::NVIC_PRIO_BITS,
                            f,
                        )
                    }
                }
            }
        )
    }

    pub fn extra_assertions(_: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![]
    }
}

pub fn pre_init_checks(app: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // check that all dispatchers exists in the `Interrupt` enumeration regardless of whether
    // they are used or not
    let interrupt = util::interrupt_ident();
    let rt_err = util::rt_err_ident();

    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = #rt_err::#interrupt::#name;));
    }

    stmts
}

pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    let interrupt = util::interrupt_ident();
    let rt_err = util::rt_err_ident();
    let device = &app.args.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);
    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

    // Unmask interrupts and set their priorities
    for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().filter_map(|task| {
        if is_exception(&task.args.binds) {
            // We do exceptions in another pass
            None
        } else {
            Some((&task.args.priority, &task.args.binds))
        }
    })) {
        let es = format!(
            "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
        );
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(
            const _: () =  if (1 << #nvic_prio_bits) < #priority as usize { ::core::panic!(#es); };
        ));

        stmts.push(quote!(
            core.NVIC.set_priority(
                #rt_err::#interrupt::#name,
                rtic::export::cortex_logical2hw(#priority, #nvic_prio_bits),
            );
        ));

        // NOTE unmask the interrupt *after* setting its priority: changing the priority of a pended
        // interrupt is implementation defined
        stmts.push(quote!(rtic::export::NVIC::unmask(#rt_err::#interrupt::#name);));
    }

    // Set exception priorities
    for (name, priority) in app.hardware_tasks.values().filter_map(|task| {
        if is_exception(&task.args.binds) {
            Some((&task.args.binds, task.args.priority))
        } else {
            None
        }
    }) {
        let es = format!(
            "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
        );
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(
            const _: () =  if (1 << #nvic_prio_bits) < #priority as usize { ::core::panic!(#es); };
        ));

        stmts.push(quote!(core.SCB.set_priority(
            rtic::export::SystemHandler::#name,
            rtic::export::cortex_logical2hw(#priority, #nvic_prio_bits),
        );));
    }

    stmts
}

pub fn architecture_specific_analysis(app: &App, _: &SyntaxAnalysis) -> parse::Result<()> {
    // Check that external (device-specific) interrupts are not named after known (Cortex-M)
    // exceptions
    for name in app.args.dispatchers.keys() {
        let name_s = name.to_string();

        match &*name_s {
            "NonMaskableInt" | "HardFault" | "MemoryManagement" | "BusFault" | "UsageFault"
            | "SecureFault" | "SVCall" | "DebugMonitor" | "PendSV" | "SysTick" => {
                return Err(parse::Error::new(
                    name.span(),
                    "Cortex-M exceptions can't be used as `extern` interrupts",
                ));
            }

            _ => {}
        }
    }

    // Check that there are enough external interrupts to dispatch the software tasks and the timer
    // queue handler
    let mut first = None;
    let priorities = app
        .software_tasks
        .iter()
        .map(|(name, task)| {
            first = Some(name);
            task.args.priority
        })
        .filter(|prio| *prio > 0)
        .collect::<HashSet<_>>();

    let need = priorities.len();
    let given = app.args.dispatchers.len();
    if need > given {
        let s = {
            format!(
                "not enough interrupts to dispatch \
                    all software tasks (need: {need}; given: {given})"
            )
        };

        // If not enough tasks and first still is None, may cause
        // "custom attribute panicked" due to unwrap on None
        return Err(parse::Error::new(first.unwrap().span(), s));
    }

    // Check that all exceptions are valid; only exceptions with configurable priorities are
    // accepted
    for (name, task) in &app.hardware_tasks {
        let name_s = task.args.binds.to_string();
        match &*name_s {
            "NonMaskableInt" | "HardFault" => {
                return Err(parse::Error::new(
                    name.span(),
                    "only exceptions with configurable priority can be used as hardware tasks",
                ));
            }

            _ => {}
        }
    }

    Ok(())
}

pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_prio_limit(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let max = if let Some(max) = analysis.max_async_prio {
        quote!(#max)
    } else {
        // No limit
        let device = &app.args.device;
        quote!(1 << #device::NVIC_PRIO_BITS)
    };

    vec![quote!(
        /// Holds the maximum priority level for use by async HAL drivers.
        #[no_mangle]
        static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8 = #max;
    )]
}
