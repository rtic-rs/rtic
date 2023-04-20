use crate::syntax::{ast::App, Context};
use core::sync::atomic::{AtomicUsize, Ordering};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, PatType};
use bindings::interrupt_ident;

const RTIC_INTERNAL: &str = "__rtic_internal";


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

/// Regroups the inputs of a task
///
/// `inputs` could be &[`input: Foo`] OR &[`mut x: i32`, `ref y: i64`]
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
            let i = Ident::new(&format!("_{i}"), Span::call_site());
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

/// Suffixed identifier
pub fn suffixed(name: &str) -> Ident {
    let span = Span::call_site();
    Ident::new(name, span)
}

pub fn static_shared_resource_ident(name: &Ident) -> Ident {
    mark_internal_name(&format!("shared_resource_{name}"))
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
