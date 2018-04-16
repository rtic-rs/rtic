//! Procedural macros of the `cortex-m-rtfm` crate
// #![deny(warnings)]
#![feature(proc_macro)]
#![recursion_limit = "128"]

#[macro_use]
extern crate failure;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate rtfm_syntax as syntax;

use proc_macro::TokenStream;
use syntax::{App, Result};

mod analyze;
mod check;
mod trans;

/// The `app!` macro, a macro used to specify the tasks and resources of a RTFM application.
///
/// The contents of this macro uses a `key: value` syntax. All the possible keys are shown below:
///
/// ``` text
/// app! {
///     device: ..,
///
///     resources: { .. },
///
///     init: { .. },
///
///     idle: { .. },
///
///     tasks: { .. },
/// }
/// ```
///
/// # `device`
///
/// The value of this key is a Rust path, like `foo::bar::baz`, that must point to a *device crate*,
/// a crate generated using `svd2rust`.
///
/// # `resources`
///
/// This key is optional. Its value is a list of `static` variables. These variables are the data
/// that can be safely accessed, modified and shared by tasks.
///
/// ``` text
/// resources: {
///     static A: bool = false;
///     static B: i32 = 0;
///     static C: [u8; 16] = [0; 16];
///     static D: Thing = Thing::new(..);
///     static E: Thing;
/// }
/// ```
///
/// The initial value of a resource can be omitted. This means that the resource will be runtime
/// initialized; these runtime initialized resources are also known as *late resources*.
///
/// If this key is omitted its value defaults to an empty list.
///
/// # `init`
///
/// This key is optional. Its value is a set of key values. All the possible keys are shown below:
///
/// ``` text
/// init: {
///     path: ..,
/// }
/// ```
///
/// ## `init.path`
///
/// This key is optional. Its value is a Rust path, like `foo::bar::baz`, that points to the
/// initialization function.
///
/// If the key is omitted its value defaults to `init`.
///
/// ## `init.resources`
///
/// This key is optional. Its value is a set of resources the `init` function *owns*. The resources
/// in this list must be a subset of the resources listed in the top `resources` key. Note that some
/// restrictions apply:
///
/// - The resources in this list can't be late resources.
/// - The resources that appear in this list can't appear in other list like `idle.resources` or
/// `tasks.$TASK.resources`
///
/// If this key is omitted its value is assumed to be an empty list.
///
/// # `idle`
///
/// This key is optional. Its value is a set of key values. All the possible keys are shown below:
///
/// ``` text
/// idle: {
///     path: ..,
///     resources: [..],
/// }
/// ```
///
/// ## `idle.path`
///
/// This key is optional. Its value is a Rust path, like `foo::bar::baz`, that points to the idle
/// loop function.
///
/// If the key is omitted its value defaults to `idle`.
///
/// ## `idle.resources`
///
/// This key is optional. Its value is a list of resources the `idle` loop has access to. The
/// resources in this list must be a subset of the resources listed in the top `resources` key.
///
/// If omitted its value defaults to an empty list.
///
/// # `tasks`
///
/// This key is optional. Its value is a list of tasks. Each task itself is a set of key value pair.
/// The full syntax is shown below:
///
/// ``` text
/// tasks: {
///     $TASK: {
///         enabled: ..,
///         path: ..,
///         priority: ..,
///         resources: [..],
///     },
/// }
/// ```
///
/// If this key is omitted its value is assumed to be an empty list.
///
/// ## `tasks.$TASK`
///
/// The key must be either a Cortex-M exception or a device specific interrupt. `PENDSV`, `SVCALL`,
/// `SYS_TICK` are considered as exceptions. All other names are assumed to be interrupts.
///
/// ## `tasks.$TASK.enabled`
///
/// This key is optional for interrupts and forbidden for exceptions. Its value must be a boolean
/// and indicates whether the interrupt will be enabled (`true`) or disabled (`false`) after `init`
/// ends and before `idle` starts.
///
/// If this key is omitted its value defaults to `true`.
///
/// ## `tasks.$TASK.path`
///
/// The value of this key is a Rust path, like `foo::bar::baz`, that points to the handler of this
/// task.
///
/// ## `tasks.$TASK.priority`
///
/// This key is optional. Its value is an integer with type `u8` that specifies the priority of this
/// task. The minimum valid priority is 1. The maximum valid priority depends on the number of the
/// NVIC priority bits the device has; if the device has 4 priority bits the maximum allowed value
/// would be 16.
///
/// If this key is omitted its value defaults to `1`.
///
/// ## `tasks.$TASK.resources`
///
/// This key is optional. Its value is a list of resources this task has access to. The resources in
/// this list must be a subset of the resources listed in the top `resources` key.
///
/// If omitted its value defaults to an empty list.
#[proc_macro]
pub fn app(ts: TokenStream) -> TokenStream {
    match run(ts) {
        Err(e) => panic!("error: {}", e),
        Ok(ts) => ts,
    }
}

fn run(ts: TokenStream) -> Result<TokenStream> {
    let app = App::parse(ts)?.check()?;
    let app = check::app(app)?;

    let ownerships = analyze::app(&app);
    let tokens = trans::app(&app, &ownerships);

    Ok(tokens.into())
}
