use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::{cell::RefCell, collections::HashSet};
use syn::{parse, parse_str, Attribute, Ident, LitStr, Path};

thread_local! {
    static PAC_PATH: RefCell<Option<String>> = RefCell::new(None);
}

pub fn interrupt_ident() -> Ident {
    let span = Span::call_site();
    Ident::new("Interrupt", span)
}

pub fn interrupt_mod(_app: &App) -> TokenStream2 {
    PAC_PATH.with(|p| {
        if let Some(s) = p.borrow().as_ref() {
            let pac: Path = parse_str(s).expect("stored pac path is valid");
            quote!(#pac::Interrupt)
        } else {
            quote!(esp32::Interrupt)
        }
    })
}

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

pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_preprocessing(app: &mut App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    let device = &app.args.device;
    let pac_str = quote!(#device).to_string();
    PAC_PATH.with(|p| *p.borrow_mut() = Some(pac_str));

    app.args.device = parse_str("crate :: __rtic_esp32_device")
        .expect("hardcoded path is valid");
    Ok(())
}

pub fn pre_init_checks(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let mut stmts: Vec<TokenStream2> = analysis
        .interrupts
        .iter()
        .map(|(priority, _)| {
            let cpu_int = match priority {
                1 => quote!(esp_hal::interrupt::CpuInterrupt::Interrupt7SoftwarePriority1),
                3 => quote!(esp_hal::interrupt::CpuInterrupt::Interrupt29SoftwarePriority3),
                p => { // lol i got gemini to generate this nice msg for me !!
                    let msg = format!(
                        "xtensa-esp32: software task dispatcher priority {p} is not supported. \
                         Only priorities 1 and 3 have dedicated CPU software interrupts \
                         (CPU int 7 / Software0 and CPU int 29 / Software1). \
                         Use a hardware task (#[task(binds = ...)]) for priority 2."
                    );
                    return quote!(compile_error!(#msg););
                }
            };
            quote!(#cpu_int.enable();)
        })
        .collect();

    let pac = PAC_PATH.with(|p| {
        p.borrow()
            .as_ref()
            .map(|s| syn::parse_str::<syn::Path>(s).expect("stored pac path is valid"))
    });
    if let Some(pac) = pac {
        for task in app.hardware_tasks.values() {
            let interrupt = &task.args.binds;
            let prio = match task.args.priority {
                1 => quote!(esp_hal::interrupt::Priority::Priority1),
                2 => quote!(esp_hal::interrupt::Priority::Priority2),
                3 => quote!(esp_hal::interrupt::Priority::Priority3),
                p => panic!("xtensa-esp32 backend: unsupported hardware task priority {p} (supported: 1, 2, 3)"),
            };
            stmts.push(quote!(
                esp_hal::interrupt::enable(#pac::Interrupt::#interrupt, #prio);
            ));
        }
    }

    stmts
}

pub fn architecture_specific_analysis(app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    for name in app.args.dispatchers.keys() {
        match name.to_string().as_str() {
            "FROM_CPU_INTR0" | "FROM_CPU_INTR1" => {}
            _ => {
                return Err(parse::Error::new(
                    name.span(),
                    "xtensa-esp32: only FROM_CPU_INTR0 and FROM_CPU_INTR1 are supported as \
                     software-task dispatchers (they map to CPU Software0 / Software1 interrupts)",
                ));
            }
        }
    }
    for (name, task) in &app.software_tasks {
        let p = task.args.priority;
        if p != 1 && p != 3 {
            return Err(parse::Error::new(
                name.span(),
                format!(
                    "xtensa-esp32: software task priority {p} is not supported; \
                     only priorities 1 (FROM_CPU_INTR0 / Software0) and \
                     3 (FROM_CPU_INTR1 / Software1) have dedicated CPU software interrupts"
                ),
            ));
        }
    }

    let priorities: HashSet<u8> = app
        .software_tasks
        .values()
        .map(|t| t.args.priority)
        .filter(|&p| p > 0)
        .collect();

    let need = priorities.len();
    let given = app.args.dispatchers.len();
    if need > given {
        let first = app.software_tasks.keys().next().unwrap();
        return Err(parse::Error::new(
            first.span(),
            format!(
                "xtensa-esp32: not enough dispatchers for software tasks \
                 (need {need}, given {given})"
            ),
        ));
    }

    Ok(())
}

pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn check_stack_overflow_before_init(
    _app: &App,
    _analysis: &CodegenAnalysis,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_entry(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    //sw interrupts are automatically cleared
    vec![]
}

pub fn async_prio_limit(_app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let max = if let Some(max) = analysis.max_async_prio {
        quote!(#max)
    } else {
        quote!(u8::MAX)
    };

    vec![quote!(
        #[no_mangle]
        static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8 = #max;
    )]
}

pub fn handler_config(
    _app: &App,
    analysis: &CodegenAnalysis,
    dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    let export_name: &str = analysis
        .interrupts
        .iter()
        .find(|(_, (name, _))| name == &dispatcher_name)
        .map(|(priority, _)| match priority {
            1 => "Software0",
            3 => "Software1",
            p => panic!("xtensa-esp32 backend: unsupported RTIC priority {p} (supported: 1, 3)"),
        })
        .unwrap_or("");

    if export_name.is_empty() {
        return vec![];
    }

    let lit = LitStr::new(export_name, Span::call_site());
    vec![quote!(#[export_name = #lit])]
}

pub fn extra_modules(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn extra_top_level(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    PAC_PATH.with(|p| {
        if let Some(s) = p.borrow().as_ref() {
            let pac: Path = parse_str(s).expect("stored pac path is valid");
            vec![quote!(
                mod __rtic_esp32_device {
                    pub use #pac::Interrupt;
                    pub type Peripherals = esp_hal::peripherals::Peripherals;
                }
            )]
        } else {
            vec![]
        }
    })
}
