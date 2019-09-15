#![deny(warnings)]

extern crate proc_macro;

use proc_macro::TokenStream;
use std::{fs, path::Path};

use rtfm_syntax::Settings;

mod analyze;
mod check;
mod codegen;
#[cfg(test)]
mod tests;

/// Attribute used to declare a RTFM application
///
/// This attribute must be applied to a `const` item of type `()`. The `const` item is effectively
/// used as a `mod` item: its value must be a block that contains items commonly found in modules,
/// like functions and `static` variables.
///
/// The `app` attribute has one mandatory argument:
///
/// - `device = <path>`. The path must point to a device crate generated using [`svd2rust`]
/// **v0.14.x** or newer.
///
/// [`svd2rust`]: https://crates.io/crates/svd2rust
///
/// and a few optional arguments:
///
/// - `peripherals = <bool>`. Indicates whether the runtime takes the device peripherals and makes
/// them available to the `init` context.
///
/// - `monotonic = <path>`. This is a path to a zero-sized structure (e.g. `struct Foo;`) that
/// implements the `Monotonic` trait. This argument must be provided to use the `schedule` API.
///
/// The items allowed in the block value of the `const` item are specified below:
///
/// # 1. `struct Resources`
///
/// This structure contains the declaration of all the resources used by the application. Each field
/// in this structure corresponds to a different resource. Each resource may optionally be given an
/// initial value using the `#[init(<value>)]` attribute. Resources with no compile-time initial
/// value as referred to as *late* resources.
///
/// # 2. `fn`
///
/// Functions must contain *one* of the following attributes: `init`, `idle` or `task`. The
/// attribute defines the role of the function in the application.
///
/// ## a. `#[init]`
///
/// This attribute indicates that the function is to be used as the *initialization function*. There
/// must be exactly one instance of the `init` attribute inside the `app` pseudo-module. The
/// signature of the `init` function must be `fn (<fn-name>::Context) [-> <fn-name>::LateResources]`
/// where `<fn-name>` is the name of the function adorned with the `#[init]` attribute.
///
/// The `init` function runs after memory (RAM) is initialized and runs with interrupts disabled.
/// Interrupts are re-enabled after `init` returns.
///
/// The `init` attribute accepts the following optional arguments:
///
/// - `resources = [resource_a, resource_b, ..]`. This is the list of resources this context has
/// access to.
///
/// - `schedule = [task_a, task_b, ..]`. This is the list of *software* tasks that this context can
/// schedule to run in the future. *IMPORTANT*: This argument is accepted only if the `monotonic`
/// argument is passed to the `#[app]` attribute.
///
/// - `spawn = [task_a, task_b, ..]`. This is the list of *software* tasks that this context can
/// immediately spawn.
///
/// The first argument of the function, `<fn-name>::Context`, is a structure that contains the
/// following fields:
///
/// - `core`. Exclusive access to core peripherals. The type of this field is [`rtfm::Peripherals`]
/// when the `schedule` API is used and [`cortex_m::Peripherals`] when it's not.
///
/// [`rtfm::Peripherals`]: ../rtfm/struct.Peripherals.html
/// [`cortex_m::Peripherals`]: https://docs.rs/cortex-m/0.6/cortex_m/peripheral/struct.Peripherals.html
///
/// - `device: <device>::Peripherals`. Exclusive access to device-specific peripherals. This
/// field is only present when the `peripherals` argument of the `#[app]` attribute is set to
/// `true`. `<device>` is the path to the device crate specified in the top `app` attribute.
///
/// - `start: <Instant>`. The `start` time of the system: `<Instant>::zero()`. `<Instant>` is the
/// `Instant` type associated to the `Monotonic` implementation specified in the top `#[app]`
/// attribute. **NOTE**: this field is only present when the `schedule` is used.
///
/// - `resources: <fn-name>::Resources`. A `struct` that contains all the resources that can be
/// accessed from this context. Each field is a different resource; each resource may appear as a
/// reference (`&[mut]-`) or as proxy structure that implements the [`rftm::Mutex`][rtfm-mutex] trait.
///
/// [rtfm-mutex]: ../rtfm/trait.Mutex.html
///
/// - `schedule: <fn-name>::Schedule`. A `struct` that can be used to schedule *software* tasks.
///
/// - `spawn: <fn-name>::Spawn`. A `struct` that can be used to spawn *software* tasks.
///
/// The return type `<fn-name>::LateResources` must only be specified when late resources, resources
/// with no initial value declared at compile time, are used. `<fn-name>::LateResources` is a
/// structure where each field corresponds to a different late resource. The
/// `<fn-name>::LateResources` value returned by the `#[init]` function is used to initialize the
/// late resources before `idle` or any task can start.
///
/// Other properties:
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// `&'static mut` references that are safe to access. For example, `static mut FOO: u32 = 0` will
/// become `FOO: &'static mut u32`.
///
/// ## b. `#[idle]`
///
/// This attribute indicates that the function is to be used as the *idle task*. There can be at
/// most once instance of the `idle` attribute inside the `app` pseudo-module. The signature of the
/// `idle` function must be `fn(<fn-name>::Context) -> !` where `<fn-name>` is the name of the
/// function adorned with the `#[idle]` attribute.
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
/// The first argument of the function, `idle::Context`, is a structure that contains the following
/// fields:
///
/// - `resources: _`. Same meaning / function as [`<init>::Context.resources`](#a-init).
///
/// - `schedule: idle::Schedule`. Same meaning / function as [`<init>::Context.schedule`](#a-init).
///
/// - `spawn: idle::Spawn`. Same meaning / function as [`<init>::Context.spawn`](#a-init).
///
/// Other properties:
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// `&'static mut` references that are safe to access. For example, `static mut FOO: u32 = 0` will
/// become `FOO: &'static mut u32`.
///
/// ## c. `#[task]`
///
/// This attribute indicates that the function is either a hardware task or a software task. The
/// signature of hardware tasks must be `fn(<fn-name>::Context)` whereas the signature of software
/// tasks must be `fn(<fn-name>::Context, <inputs>)`. `<fn-name>` refers to the name of the function
/// adorned with the `#[task]` attribute.
///
/// The `task` attribute accepts the following optional arguments.
///
/// - `binds = <interrupt-name>`. Binds this task to a particular interrupt. When this argument is
/// present the task is treated as a hardware task; when it's omitted the task treated is treated as
/// a software task.
///
/// - `priority = <integer>`. This is the static priority of the exception handler. The value must
/// be in the range `1..=(1 << <device-path>::NVIC_PRIO_BITS)` where `<device-path>` is the path to
/// the device crate specified in the top `app` attribute. If this argument is omitted the priority
/// is assumed to be 1.
///
/// - `resources = (..)`. Same meaning / function as [`#[init].resources`](#a-init).
///
/// - `schedule = (..)`. Same meaning / function as [`#[init].schedule`](#a-init).
///
/// - `spawn = (..)`. Same meaning / function as [`#[init].spawn`](#a-init).
///
/// The first argument of the function, `<fn-name>::Context`, is a structure that contains the
/// following fields:
///
/// - `start: <Instant>`. For hardware tasks this is the time at which this handler started
/// executing. For software tasks this is the time at which the task was scheduled to run. **NOTE**:
/// only present when the `schedule` API is used.
///
/// - `resources: _`. Same meaning / function as [`<init>::Context.resources`](#a-init).
///
/// - `schedule: <exception-name>::Schedule`. Same meaning / function as
/// [`<init>::Context.schedule`](#a-init).
///
/// - `spawn: <exception-name>::Spawn`.  Same meaning / function as
/// [`<init>::Context.spawn`](#a-init).
///
/// Other properties / constraints:
///
/// - The `static mut` variables declared at the beginning of this function will be transformed into
/// *non*-static `&mut` references that are safe to access. For example, `static mut FOO: u32 = 0`
/// will become `FOO: &mut u32`.
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
/// Attributes can be applied to the functions inside this block. These attributes will be forwarded
/// to the interrupt handlers generated by the `app` attribute.
#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut settings = Settings::default();
    settings.optimize_priorities = true;
    settings.parse_binds = true;
    settings.parse_cores = cfg!(feature = "heterogeneous") || cfg!(feature = "homogeneous");
    settings.parse_extern_interrupt = true;
    settings.parse_schedule = true;

    let (app, analysis) = match rtfm_syntax::parse(args, input, settings) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    let extra = match check::app(&app, &analysis) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    let analysis = analyze::app(analysis, &app);

    let ts = codegen::app(&app, &analysis, &extra);

    // Try to write the expanded code to disk
    if Path::new("target").exists() {
        fs::write("target/rtfm-expansion.rs", ts.to_string()).ok();
    }

    ts.into()
}
