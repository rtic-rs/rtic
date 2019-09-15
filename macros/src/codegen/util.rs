use core::sync::atomic::{AtomicUsize, Ordering};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use rtfm_syntax::{ast::App, Context, Core};
use syn::{Attribute, Ident, LitInt, PatType};

use crate::check::Extra;

/// Turns `capacity` into an unsuffixed integer literal
pub fn capacity_literal(capacity: u8) -> LitInt {
    LitInt::new(&capacity.to_string(), Span::call_site())
}

/// Turns `capacity` into a type-level (`typenum`) integer
pub fn capacity_typenum(capacity: u8, round_up_to_power_of_two: bool) -> TokenStream2 {
    let capacity = if round_up_to_power_of_two {
        capacity.checked_next_power_of_two().expect("UNREACHABLE")
    } else {
        capacity
    };

    let ident = Ident::new(&format!("U{}", capacity), Span::call_site());

    quote!(rtfm::export::consts::#ident)
}

/// Generates a `#[cfg(core = "0")]` attribute if we are in multi-core mode
pub fn cfg_core(core: Core, cores: u8) -> Option<TokenStream2> {
    if cores == 1 {
        None
    } else if cfg!(feature = "heterogeneous") {
        let core = core.to_string();
        Some(quote!(#[cfg(core = #core)]))
    } else {
        None
    }
}

/// Identifier for the free queue
///
/// There may be more than one free queue per task because we need one for each sender core so we
/// include the sender (e.g. `S0`) in the name
pub fn fq_ident(task: &Ident, sender: Core) -> Ident {
    Ident::new(
        &format!("{}_S{}_FQ", task.to_string(), sender),
        Span::call_site(),
    )
}

/// Generates a `Mutex` implementation
pub fn impl_mutex(
    extra: &Extra,
    cfgs: &[Attribute],
    cfg_core: Option<&TokenStream2>,
    resources_prefix: bool,
    name: &Ident,
    ty: TokenStream2,
    ceiling: u8,
    ptr: TokenStream2,
) -> TokenStream2 {
    let (path, priority) = if resources_prefix {
        (quote!(resources::#name), quote!(self.priority()))
    } else {
        (quote!(#name), quote!(self.priority))
    };

    let device = extra.device;
    quote!(
        #(#cfgs)*
        #cfg_core
        impl<'a> rtfm::Mutex for #path<'a> {
            type T = #ty;

            #[inline(always)]
            fn lock<R>(&mut self, f: impl FnOnce(&mut #ty) -> R) -> R {
                /// Priority ceiling
                const CEILING: u8 = #ceiling;

                unsafe {
                    rtfm::export::lock(
                        #ptr,
                        #priority,
                        CEILING,
                        #device::NVIC_PRIO_BITS,
                        f,
                    )
                }
            }
        }
    )
}

/// Generates an identifier for a cross-initialization barrier
pub fn init_barrier(initializer: Core) -> Ident {
    Ident::new(&format!("IB{}", initializer), Span::call_site())
}

/// Generates an identifier for the `INPUTS` buffer (`spawn` & `schedule` API)
pub fn inputs_ident(task: &Ident, sender: Core) -> Ident {
    Ident::new(&format!("{}_S{}_INPUTS", task, sender), Span::call_site())
}

/// Generates an identifier for the `INSTANTS` buffer (`schedule` API)
pub fn instants_ident(task: &Ident, sender: Core) -> Ident {
    Ident::new(&format!("{}_S{}_INSTANTS", task, sender), Span::call_site())
}

pub fn interrupt_ident(core: Core, cores: u8) -> Ident {
    let span = Span::call_site();
    if cores == 1 {
        Ident::new("Interrupt", span)
    } else {
        Ident::new(&format!("Interrupt_{}", core), span)
    }
}

/// Whether `name` is an exception with configurable priority
pub fn is_exception(name: &Ident) -> bool {
    let s = name.to_string();

    match &*s {
        "MemoryManagement" | "BusFault" | "UsageFault" | "SecureFault" | "SVCall"
        | "DebugMonitor" | "PendSV" | "SysTick" => true,

        _ => false,
    }
}

/// Generates a pre-reexport identifier for the "late resources" struct
pub fn late_resources_ident(init: &Ident) -> Ident {
    Ident::new(
        &format!("{}LateResources", init.to_string()),
        Span::call_site(),
    )
}

fn link_section_index() -> usize {
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    INDEX.fetch_add(1, Ordering::Relaxed)
}

pub fn link_section(section: &str, core: Core) -> Option<TokenStream2> {
    if cfg!(feature = "homogeneous") {
        let section = format!(".{}_{}.rtfm{}", section, core, link_section_index());
        Some(quote!(#[link_section = #section]))
    } else {
        None
    }
}

// NOTE `None` means in shared memory
pub fn link_section_uninit(core: Option<Core>) -> Option<TokenStream2> {
    let section = if let Some(core) = core {
        let index = link_section_index();

        if cfg!(feature = "homogeneous") {
            format!(".uninit_{}.rtfm{}", core, index)
        } else {
            format!(".uninit.rtfm{}", index)
        }
    } else {
        if cfg!(feature = "heterogeneous") {
            // `#[shared]` attribute sets the linker section
            return None;
        }

        format!(".uninit.rtfm{}", link_section_index())
    };

    Some(quote!(#[link_section = #section]))
}

/// Generates a pre-reexport identifier for the "locals" struct
pub fn locals_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init(core) => app.inits[&core].name.to_string(),
        Context::Idle(core) => app.idles[&core].name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("Locals");

    Ident::new(&s, Span::call_site())
}

/// Generates an identifier for a rendezvous barrier
pub fn rendezvous_ident(core: Core) -> Ident {
    Ident::new(&format!("RV{}", core), Span::call_site())
}

// Regroups the inputs of a task
//
// `inputs` could be &[`input: Foo`] OR &[`mut x: i32`, `ref y: i64`]
pub fn regroup_inputs(
    inputs: &[PatType],
) -> (
    // args e.g. &[`_0`],  &[`_0: i32`, `_1: i64`]
    Vec<TokenStream2>,
    // tupled e.g. `_0`, `(_0, _1)`
    TokenStream2,
    // untupled e.g. &[`_0`], &[`_0`, `_1`]
    Vec<TokenStream2>,
    // ty e.g. `Foo`, `(i32, i64)`
    TokenStream2,
) {
    if inputs.len() == 1 {
        let ty = &inputs[0].ty;

        (
            vec![quote!(_0: #ty)],
            quote!(_0),
            vec![quote!(_0)],
            quote!(#ty),
        )
    } else {
        let mut args = vec![];
        let mut pats = vec![];
        let mut tys = vec![];

        for (i, input) in inputs.iter().enumerate() {
            let i = Ident::new(&format!("_{}", i), Span::call_site());
            let ty = &input.ty;

            args.push(quote!(#i: #ty));

            pats.push(quote!(#i));

            tys.push(quote!(#ty));
        }

        let tupled = {
            let pats = pats.clone();
            quote!((#(#pats,)*))
        };
        let ty = quote!((#(#tys,)*));
        (args, tupled, pats, ty)
    }
}

/// Generates a pre-reexport identifier for the "resources" struct
pub fn resources_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init(core) => app.inits[&core].name.to_string(),
        Context::Idle(core) => app.idles[&core].name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("Resources");

    Ident::new(&s, Span::call_site())
}

/// Generates an identifier for a ready queue
///
/// Each core may have several task dispatchers, one for each priority level. Each task dispatcher
/// in turn may use more than one ready queue because the queues are SPSC queues so one is needed
/// per sender core.
pub fn rq_ident(receiver: Core, priority: u8, sender: Core) -> Ident {
    Ident::new(
        &format!("R{}_P{}_S{}_RQ", receiver, priority, sender),
        Span::call_site(),
    )
}

/// Generates an identifier for a "schedule" function
///
/// The methods of the `Schedule` structs invoke these functions. As one task may be `schedule`-ed
/// by different cores we need one "schedule" function per possible task-sender pair
pub fn schedule_ident(name: &Ident, sender: Core) -> Ident {
    Ident::new(
        &format!("schedule_{}_S{}", name.to_string(), sender),
        Span::call_site(),
    )
}

/// Generates an identifier for the `enum` of `schedule`-able tasks
pub fn schedule_t_ident(core: Core) -> Ident {
    Ident::new(&format!("T{}", core), Span::call_site())
}

/// Generates an identifier for a cross-spawn barrier
pub fn spawn_barrier(receiver: Core) -> Ident {
    Ident::new(&format!("SB{}", receiver), Span::call_site())
}

/// Generates an identifier for a "spawn" function
///
/// The methods of the `Spawn` structs invoke these functions. As one task may be `spawn`-ed by
/// different cores we need one "spawn" function per possible task-sender pair
pub fn spawn_ident(name: &Ident, sender: Core) -> Ident {
    Ident::new(
        &format!("spawn_{}_S{}", name.to_string(), sender),
        Span::call_site(),
    )
}

/// Generates an identifier for the `enum` of `spawn`-able tasks
///
/// This identifier needs the same structure as the `RQ` identifier because there's one ready queue
/// for each of these `T` enums
pub fn spawn_t_ident(receiver: Core, priority: u8, sender: Core) -> Ident {
    Ident::new(
        &format!("R{}_P{}_S{}_T", receiver, priority, sender),
        Span::call_site(),
    )
}

pub fn suffixed(name: &str, core: u8) -> Ident {
    let span = Span::call_site();

    if cfg!(feature = "homogeneous") {
        Ident::new(&format!("{}_{}", name, core), span)
    } else {
        Ident::new(name, span)
    }
}

/// Generates an identifier for a timer queue
///
/// At most there's one timer queue per core
pub fn tq_ident(core: Core) -> Ident {
    Ident::new(&format!("TQ{}", core), Span::call_site())
}
