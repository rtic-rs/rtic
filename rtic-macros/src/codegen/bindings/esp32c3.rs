use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
    codegen::util,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse, Attribute, Ident};


//#[cfg((feature = esp32c3)]
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
                /// Priority ceiling
                const CEILING: u8 = #ceiling;
                unsafe {
                    rtic::export::lock(
                        #ptr,
                        CEILING,
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


pub fn pre_init_checks(app: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];
    // check that all dispatchers exists in the `Interrupt` enumeration regardless of whether
    // they are used or not
    let rt_err = util::rt_err_ident();

    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = #rt_err::Interrupt::#name;));
    }
    stmts
}
pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];
    let rt_err = util::rt_err_ident();
    let max_prio:usize = 15; //unfortunately this is not part of pac, but we know that max prio is 15.
    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));
    // Unmask interrupts and set their priorities
    for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().filter_map(|task| {
        Some((&task.args.priority, &task.args.binds))
    })){
        let es = format!(
            "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
        );
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(
            const _: () =  if (#max_prio) <= #priority as usize { ::core::panic!(#es); };
        ));    
        //hal enables interrupt and sets prio simultaneously
        stmts.push(quote!(
            rtic::export::hal_interrupt::enable(
                #rt_err::Interrupt::#name, //interrupt struct
                rtic::export::int_to_prio(#priority) //interrupt priority object            
            );
        ));
    }
    stmts
}


pub fn architecture_specific_analysis(_app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    Ok(())
}

pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}