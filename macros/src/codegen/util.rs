use core::sync::atomic::{AtomicUsize, Ordering};

use crate::syntax::{ast::App, Context};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Attribute, Ident};

const RTIC_INTERNAL: &str = "__rtic_internal";

/// Generates a `Mutex` implementation
pub fn impl_mutex(
    app: &App,
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
    let masks_name = priority_masks_ident();
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
                        &#masks_name,
                        f,
                    )
                }
            }
        }
    )
}

/// Generates an identifier for the `EXECUTOR_RUN` atomics (`async` API)
pub fn executor_run_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{task}_EXECUTOR_RUN"))
}

pub fn interrupt_ident() -> Ident {
    let span = Span::call_site();
    Ident::new("interrupt", span)
}

/// Whether `name` is an exception with configurable priority
pub fn is_exception(name: &Ident) -> bool {
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

/// Mark a name as internal
pub fn mark_internal_name(name: &str) -> Ident {
    Ident::new(&format!("{RTIC_INTERNAL}_{name}"), Span::call_site())
}

/// Generate an internal identifier for tasks
pub fn internal_task_ident(task: &Ident, ident_name: &str) -> Ident {
    mark_internal_name(&format!("{task}_{ident_name}"))
}

fn link_section_index() -> usize {
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    INDEX.fetch_add(1, Ordering::Relaxed)
}

/// Add `link_section` attribute
pub fn link_section_uninit() -> TokenStream2 {
    let section = format!(".uninit.rtic{}", link_section_index());

    quote!(#[link_section = #section])
}

/// Get the ident for the name of the task
pub fn get_task_name(ctxt: Context, app: &App) -> Ident {
    let s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app
            .idle
            .as_ref()
            .expect("RTIC-ICE: unable to find idle name")
            .name
            .to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    Ident::new(&s, Span::call_site())
}

/// Generates a pre-reexport identifier for the "shared resources" struct
pub fn shared_resources_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app
            .idle
            .as_ref()
            .expect("RTIC-ICE: unable to find idle name")
            .name
            .to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("SharedResources");

    mark_internal_name(&s)
}

/// Generates a pre-reexport identifier for the "local resources" struct
pub fn local_resources_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app
            .idle
            .as_ref()
            .expect("RTIC-ICE: unable to find idle name")
            .name
            .to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("LocalResources");

    mark_internal_name(&s)
}

/// Generates an identifier for a ready queue, async task version
pub fn rq_async_ident(async_task_name: &Ident) -> Ident {
    mark_internal_name(&format!("ASYNC_TASK_{async_task_name}_RQ"))
}

/// Suffixed identifier
pub fn suffixed(name: &str) -> Ident {
    let span = Span::call_site();
    Ident::new(name, span)
}

pub fn static_shared_resource_ident(name: &Ident) -> Ident {
    mark_internal_name(&format!("shared_resource_{name}"))
}

/// Generates an Ident for the number of 32 bit chunks used for Mask storage.
pub fn priority_mask_chunks_ident() -> Ident {
    mark_internal_name("MASK_CHUNKS")
}

pub fn priority_masks_ident() -> Ident {
    mark_internal_name("MASKS")
}

pub fn static_local_resource_ident(name: &Ident) -> Ident {
    mark_internal_name(&format!("local_resource_{name}"))
}

pub fn declared_static_local_resource_ident(name: &Ident, task_name: &Ident) -> Ident {
    mark_internal_name(&format!("local_{task_name}_{name}"))
}

pub fn need_to_lock_ident(name: &Ident) -> Ident {
    Ident::new(&format!("{name}_that_needs_to_be_locked"), name.span())
}

pub fn zero_prio_dispatcher_ident() -> Ident {
    Ident::new("__rtic_internal_async_0_prio_dispatcher", Span::call_site())
}

/// The name to get better RT flag errors
pub fn rt_err_ident() -> Ident {
    Ident::new(
        "you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml",
        Span::call_site(),
    )
}
