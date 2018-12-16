#![deny(warnings)]
#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod analyze;
mod check;
mod codegen;
mod syntax;

/// Attribute used to declare a RTFM application
///
/// This attribute must be applied to a `const` item of type `()`. The `const` item is effectively
/// used as a `mod` item: its value must be a block that contains items commonly found in modules,
/// like functions and `static` variables.
///
/// The `app` attribute has one mandatory argument:
///
/// - `device = <path>`. The path must point to a device crate generated using [`svd2rust`]
/// **v0.14.x**.
///
/// [`svd2rust`]: https://crates.io/crates/svd2rust
///
/// The items allowed in the block value of the `const` item are specified below:
///
/// # 1. `static [mut]` variables
///
/// These variables are used as *resources*. Resources can be owned by tasks or shared between them.
/// Tasks can get `&mut` (exclusives) references to `static mut` resources, but only `&` (shared)
/// references to `static` resources. Lower priority tasks will need a [`lock`] to get a `&mut`
/// reference to a `static mut` resource shared with higher priority tasks.
///
/// [`lock`]: ../rtfm/trait.Mutex.html#method.lock
///
/// `static mut` resources that are shared by tasks that run at *different* priorities need to
/// implement the [`Send`] trait. Similarly, `static` resources that are shared by tasks that run at
/// *different* priorities need to implement the [`Sync`] trait.
///
/// [`Send`]: https://doc.rust-lang.org/core/marker/trait.Send.html
/// [`Sync`]: https://doc.rust-lang.org/core/marker/trait.Sync.html
///
/// Resources can be initialized at runtime by assigning them `()` (the unit value) as their initial
/// value in their declaration. These "late" resources need to be initialized an the end of the
/// `init` function.
///
/// The `app` attribute will inject a `resources` module in the root of the crate. This module
/// contains proxy `struct`s that implement the [`Mutex`] trait. The `struct` are named after the
/// `static mut` resources. For example, `static mut FOO: u32 = 0` will map to a `resources::FOO`
/// `struct` that implements the `Mutex<Data = u32>` trait.
///
/// [`Mutex`]: ../rtfm/trait.Mutex.html
///
/// # 2. `fn`
///
/// Functions must contain *one* of the following attributes: `init`, `idle`, `interrupt`,
/// `exception` or `task`. The attribute defines the role of the function in the application.
///
/// ## a. `#[init]`
///
/// This attribute indicates that the function is to be used as the *initialization function*. There
/// must be exactly one instance of the `init` attribute inside the `app` pseudo-module. The
/// signature of the `init` function must be `[unsafe] fn ()`.
///
/// The `init` function runs after memory (RAM) is initialized and runs with interrupts disabled.
/// Interrupts are re-enabled after `init` returns.
///
/// The `init` attribute accepts the following optional arguments:
///
/// - `resources = [RESOURCE_A, RESOURCE_B, ..]`. This is the list of resources this function has
/// access to.
///
/// - `schedule = [task_a, task_b, ..]`. This is the list of *software* tasks that this function can
/// schedule to run in the future. *IMPORTANT*: This argument is accepted only if the `timer-queue`
/// feature has been enabled.
///
/// - `spawn = [task_a, task_b, ..]`. This is the list of *software* tasks that this function can
/// immediately spawn.
///
/// The `app` attribute will injected a *context* into this function that comprises the following
/// variables:
///
/// - `core: rtfm::Peripherals`. Exclusive access to core peripherals. See [`rtfm::Peripherals`] for
/// more details.
///
/// [`rtfm::Peripherals`]: ../rtfm/struct.Peripherals.html
///
/// - `device: <device-path>::Peripherals`. Exclusive access to device-specific peripherals.
/// `<device-path>` is the path to the device crate declared in the top `app` attribute.
///
/// - `start: rtfm::Instant`. The `start` time of the system: `Instant(0 /* cycles */)`. **NOTE**:
/// only present if the `timer-queue` feature is enabled.
///
/// - `resources: _`. An opaque `struct` that contains all the resources assigned to this function.
/// The resource maybe appear by value (`impl Singleton`), by references (`&[mut]`) or by proxy
/// (`impl Mutex`).
///
/// - `schedule: init::Schedule`. A `struct` that can be used to schedule *software* tasks.
/// **NOTE**: only present if the `timer-queue` feature is enabled.
///
/// - `spawn: init::Spawn`. A `struct` that can be used to spawn *software* tasks.
///
/// Other properties / constraints:
///
/// - The `init` function can **not** be called from software.
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// `&'static mut` references that are safe to access. For example, `static mut FOO: u32 = 0` will
/// become `FOO: &'static mut u32`.
///
/// - Assignments (e.g. `FOO = 0`) at the end of this function can be used to initialize *late*
/// resources.
///
/// ## b. `#[idle]`
///
/// This attribute indicates that the function is to be used as the *idle task*. There can be at
/// most once instance of the `idle` attribute inside the `app` pseudo-module. The signature of the
/// `idle` function must be `fn() -> !`.
///
/// The `idle` task is a special task that always runs in the background. The `idle` task runs at
/// the lowest priority of `0`. If the `idle` task is not defined then the runtime sets the
/// [SLEEPONEXIT] bit after executing `init`.
///
/// [SLEEPONEXIT]: https://developer.arm.com/products/architecture/cpu-architecture/m-profile/docs/100737/0100/power-management/sleep-mode/sleep-on-exit-bit
///
/// The `idle` attribute accepts the following optional arguments:
///
/// - `resources = (..)`. Same meaning / function as [`#[init].resources`](#a-init).
///
/// - `schedule = (..)`. Same meaning / function as [`#[init].schedule`](#a-init).
///
/// - `spawn = (..)`. Same meaning / function as [`#[init].spawn`](#a-init).
///
/// The `app` attribute will injected a *context* into this function that comprises the following
/// variables:
///
/// - `resources: _`. Same meaning / function as [`init.resources`](#a-init).
///
/// - `schedule: idle::Schedule`. Same meaning / function as [`init.schedule`](#a-init).
///
/// - `spawn: idle::Spawn`. Same meaning / function as [`init.spawn`](#a-init).
///
/// Other properties / constraints:
///
/// - The `idle` function can **not** be called from software.
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// `&'static mut` references that are safe to access. For example, `static mut FOO: u32 = 0` will
/// become `FOO: &'static mut u32`.
///
/// ## c. `#[exception]`
///
/// This attribute indicates that the function is to be used as an *exception handler*, a type of
/// hardware task. The signature of `exception` handlers must be `[unsafe] fn()`.
///
/// The name of the function must match one of the Cortex-M exceptions that has [configurable
/// priority][system-handler].
///
/// [system-handler]: ../cortex_m/peripheral/scb/enum.SystemHandler.html
///
/// The `exception` attribute accepts the following optional arguments.
///
/// - `priority = <integer>`. This is the static priority of the exception handler. The value must
/// be in the range `1..=(1 << <device-path>::NVIC_PRIO_BITS)` where `<device-path>` is the path to
/// the device crate declared in the top `app` attribute. If this argument is omitted the priority
/// is assumed to be 1.
///
/// - `resources = (..)`. Same meaning / function as [`#[init].resources`](#a-init).
///
/// - `schedule = (..)`. Same meaning / function as [`#[init].schedule`](#a-init).
///
/// - `spawn = (..)`. Same meaning / function as [`#[init].spawn`](#a-init).
///
/// The `app` attribute will injected a *context* into this function that comprises the following
/// variables:
///
/// - `start: rtfm::Instant`. The time at which this handler started executing. **NOTE**: only
/// present if the `timer-queue` feature is enabled.
///
/// - `resources: _`. Same meaning / function as [`init.resources`](#a-init).
///
/// - `schedule: <exception-name>::Schedule`. Same meaning / function as [`init.schedule`](#a-init).
///
/// - `spawn: <exception-name>::Spawn`.  Same meaning / function as [`init.spawn`](#a-init).
///
/// Other properties / constraints:
///
/// - `exception` handlers can **not** be called from software.
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// `&mut` references that are safe to access. For example, `static mut FOO: u32 = 0` will
/// become `FOO: &mut u32`.
///
/// ## d. `#[interrupt]`
///
/// This attribute indicates that the function is to be used as an *interrupt handler*, a type of
/// hardware task. The signature of `interrupt` handlers must be `[unsafe] fn()`.
///
/// The name of the function must match one of the device specific interrupts. See your device crate
/// documentation (`Interrupt` enum) for more details.
///
/// The `interrupt` attribute accepts the following optional arguments.
///
/// - `priority = (..)`. Same meaning / function as [`#[exception].priority`](#b-exception).
///
/// - `resources = (..)`. Same meaning / function as [`#[init].resources`](#a-init).
///
/// - `schedule = (..)`. Same meaning / function as [`#[init].schedule`](#a-init).
///
/// - `spawn = (..)`. Same meaning / function as [`#[init].spawn`](#a-init).
///
/// The `app` attribute will injected a *context* into this function that comprises the following
/// variables:
///
/// - `start: rtfm::Instant`. Same meaning / function as [`exception.start`](#b-exception).
///
/// - `resources: _`. Same meaning / function as [`init.resources`](#a-init).
///
/// - `schedule: <interrupt-name>::Schedule`. Same meaning / function as [`init.schedule`](#a-init).
///
/// - `spawn: <interrupt-name>::Spawn`.  Same meaning / function as [`init.spawn`](#a-init).
///
/// Other properties / constraints:
///
/// - `interrupt` handlers can **not** be called from software, but they can be [`pend`]-ed by the
/// software from any context.
///
/// [`pend`]: ../rtfm/fn.pend.html
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// `&mut` references that are safe to access. For example, `static mut FOO: u32 = 0` will
/// become `FOO: &mut u32`.
///
/// ## e. `#[task]`
///
/// This attribute indicates that the function is to be used as a *software task*. The signature of
/// software `task`s must be `[unsafe] fn(<inputs>)`.
///
/// The `task` attribute accepts the following optional arguments.
///
/// - `capacity = <integer>`. The maximum number of instances of this task that can be queued onto
/// the task scheduler for execution. The value must be in the range `1..=255`. If the `capacity`
/// argument is omitted then the capacity will be inferred.
///
/// - `priority = <integer>`. Same meaning / function as [`#[exception].priority`](#b-exception).
///
/// - `resources = (..)`. Same meaning / function as [`#[init].resources`](#a-init).
///
/// - `schedule = (..)`. Same meaning / function as [`#[init].schedule`](#a-init).
///
/// - `spawn = (..)`. Same meaning / function as [`#[init].spawn`](#a-init).
///
/// The `app` attribute will injected a *context* into this function that comprises the following
/// variables:
///
/// - `scheduled: rtfm::Instant`. The time at which this task was scheduled to run. **NOTE**: Only
/// present if `timer-queue` is enabled.
///
/// - `resources: _`. Same meaning / function as [`init.resources`](#a-init).
///
/// - `schedule: <interrupt-name>::Schedule`. Same meaning / function as [`init.schedule`](#a-init).
///
/// - `spawn: <interrupt-name>::Spawn`.  Same meaning / function as [`init.spawn`](#a-init).
///
/// Other properties / constraints:
///
/// - Software `task`s can **not** be called from software, but they can be `spawn`-ed and
/// `schedule`-d by the software from any context.
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// `&mut` references that are safe to access. For example, `static mut FOO: u32 = 0` will
/// become `FOO: &mut u32`.
///
/// # 3. `extern` block
///
/// This `extern` block contains a list of interrupts which are *not* used by the application as
/// hardware tasks. These interrupts will be used to dispatch software tasks. Each interrupt will be
/// used to dispatch *multiple* software tasks *at the same priority level*.
///
/// This `extern` block must only contain functions with signature `fn ()`. The names of these
/// functions must match the names of the target device interrupts.
///
/// Importantly, attributes can be applied to the functions inside this block. These attributes will
/// be forwarded to the interrupt handlers generated by the `app` attribute.
#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse
    let args = parse_macro_input!(args as syntax::AppArgs);
    let items = parse_macro_input!(input as syntax::Input).items;

    let app = match syntax::App::parse(items, args) {
        Err(e) => return e.to_compile_error().into(),
        Ok(app) => app,
    };

    // Check the specification
    if let Err(e) = check::app(&app) {
        return e.to_compile_error().into();
    }

    // Ceiling analysis
    let analysis = analyze::app(&app);

    // Code generation
    codegen::app(&app, &analysis).into()
}
