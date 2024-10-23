use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{
        analyze::Analysis as SyntaxAnalysis,
        ast::{App, Dispatcher},
    },
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::{collections::HashSet, vec};
use syn::{parse, Attribute, Ident};

/// Utility function to get the SLIC interrupt module.
pub fn interrupt_ident() -> Ident {
    let span = Span::call_site();
    Ident::new("SoftwareInterrupt", span)
}

pub fn interrupt_mod(_app: &App) -> TokenStream2 {
    let interrupt = interrupt_ident();
    quote!(slic::#interrupt)
}

/// This macro implements the [`rtic::Mutex`] trait for shared resources using the SLIC.
#[allow(clippy::too_many_arguments)]
pub fn impl_mutex(
    _app: &App,
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

    quote!(
        #(#cfgs)*
        impl<'a> rtic::Mutex for #path<'a> {
            type T = #ty;

            #[inline(always)]
            fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {

                const CEILING: u8 = #ceiling;

                unsafe {
                    rtic::export::lock(#ptr, CEILING, f)
                }
            }
        }
    )
}

/// This macro is used to define additional compile-time assertions in case the platform needs it.
/// The Cortex-M implementations do not use it. Thus, we think we do not need it either.
pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_preprocessing(app: &mut App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    app.args.core = false; // RISC-V SLIC is not compatible with using core peripherals
    if !app.args.dispatchers.is_empty() {
        return Err(parse::Error::new(
            Span::call_site(),
            "this backend does not support explicit interrupt dispatchers; remove the `dispatchers` argument from `#[app]`",
        ));
    }

    // Compute the number of handlers we need to dispatch the software tasks
    let soft_priorities = app
        .software_tasks
        .iter()
        .map(|(_, task)| task.args.priority)
        .filter(|prio| *prio > 0)
        .collect::<HashSet<_>>();

    for i in 0..soft_priorities.len() {
        let dispatcher_ident = Ident::new(&format!("__RTICDispatcher{}", i), Span::call_site());
        app.args
            .dispatchers
            .insert(dispatcher_ident, Dispatcher { attrs: vec![] });
    }

    Ok(())
}

/// This macro is used to check at run-time that all the interruption dispatchers exist.
pub fn pre_init_checks(app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts: Vec<TokenStream2> = vec![];
    let int_mod = interrupt_mod(app);

    // check that all dispatchers exists in the `slic::Interrupt` enumeration
    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = #int_mod::#name;));
    }

    stmts
}

/// This macro must perform all the required operations to activate the
/// interrupt sources with their corresponding priority level.
pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // First, we reset and disable all the interrupt controllers
    stmts.push(quote!(rtic::export::clear_interrupts();));

    // Then, we set the corresponding priorities
    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));
    for (&p, name) in interrupt_ids.chain(
        app.hardware_tasks
            .values()
            .map(|task| (&task.args.priority, &task.args.binds)),
    ) {
        stmts.push(quote!(
            rtic::export::set_priority(slic::SoftwareInterrupt::#name, #p);
        ));
    }
    // Finally, we activate the interrupts
    stmts.push(quote!(rtic::export::set_interrupts();));
    stmts
}

/// Any additional checks that depend on the system architecture.
pub fn architecture_specific_analysis(app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    // Check that there are enough external interrupts to dispatch the software tasks and the timer queue handler
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

        return Err(parse::Error::new(first.unwrap().span(), s));
    }
    #[cfg(feature = "riscv-clint")]
    if app.args.backend.is_none() {
        return Err(parse::Error::new(
            Span::call_site(),
            "CLINT requires backend-specific configuration",
        ));
    }

    Ok(())
}

/// Macro to add statements to be executed at the beginning of all the interrupt handlers.
pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

/// Macro to add statements to be executed at the end of all the interrupt handlers.
pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn check_stack_overflow_before_init(
    _app: &App,
    _analysis: &CodegenAnalysis,
) -> Vec<TokenStream2> {
    vec![quote!(
        // Check for stack overflow using symbols from `risc-v-rt`.
        extern "C" {
            pub static _stack_start: u32;
            pub static _ebss: u32;
        }

        let stack_start = &_stack_start as *const _ as u32;
        let ebss = &_ebss as *const _ as u32;

        if stack_start > ebss {
            // No flip-link usage, check the SP for overflow.
            if rtic::export::read_sp() <= ebss {
                panic!("Stack overflow after allocating executors");
            }
        }
    )]
}

pub fn async_entry(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

/// Macro to define a maximum priority level for async tasks.
pub fn async_prio_limit(_app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let max = if let Some(max) = analysis.max_async_prio {
        quote!(#max)
    } else {
        quote!(u8::MAX) // No limit
    };

    vec![quote!(
        /// Holds the maximum priority level for use by async HAL drivers.
        #[no_mangle]
        static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8 = #max;
    )]
}

pub fn handler_config(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

/// The SLIC requires us to call to the [`riscv_rtic::codegen`] macro to generate
/// the appropriate SLIC structure, interrupt enumerations, etc.
pub fn extra_modules(app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    let hw_slice: Vec<_> = app
        .hardware_tasks
        .values()
        .map(|task| &task.args.binds)
        .collect();
    let sw_slice: Vec<_> = app.args.dispatchers.keys().collect();

    let swi_slice: Vec<_> = hw_slice.iter().chain(sw_slice.iter()).collect();

    let device = &app.args.device;

    stmts.push(quote!(
        use rtic::export::riscv_slic;
    ));
    let slic = quote! {rtic::export::riscv_slic};

    match () {
        #[cfg(feature = "riscv-clint")]
        () => {
            let hart_id = &app.args.backend.as_ref().unwrap().hart_id;
            stmts.push(quote!(rtic::export::codegen!(slic = #slic, pac = #device, swi = [#(#swi_slice,)*], backend = [hart_id = #hart_id]);));
        }
        #[cfg(feature = "riscv-mecall")]
        () => {
            stmts.push(quote!(rtic::export::codegen!(slic = #slic, pac = #device, swi = [#(#swi_slice,)*]);));
        }
    }

    stmts
}
