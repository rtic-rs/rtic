#![deny(warnings)]

use proc_macro::TokenStream;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use proc_macro2::Span;
use quote::quote;
use rand::{Rng, SeedableRng};
use syn::{parse_quote, ArgCaptured, Attribute, Ident, IntSuffix, LitInt};

use crate::{
    analyze::{Analysis, Ownership},
    syntax::{App, Idents, Static},
};

// NOTE to avoid polluting the user namespaces we map some identifiers to pseudo-hygienic names.
// In some instances we also use the pseudo-hygienic names for safety, for example the user should
// not modify the priority field of resources.
type Aliases = HashMap<Ident, Ident>;

struct Context {
    // Alias
    #[cfg(feature = "timer-queue")]
    baseline: Ident,
    dispatchers: HashMap<u8, Dispatcher>,
    // Alias (`fn`)
    idle: Ident,
    // Alias (`fn`)
    init: Ident,
    // Alias
    priority: Ident,
    // For non-singletons this maps the resource name to its `static mut` variable name
    statics: Aliases,
    /// Task -> Alias (`struct`)
    resources: HashMap<Kind, Resources>,
    // Alias (`enum`)
    schedule_enum: Ident,
    // Task -> Alias (`fn`)
    schedule_fn: Aliases,
    tasks: HashMap<Ident, Task>,
    // Alias (`struct` / `static mut`)
    timer_queue: Ident,
}

struct Dispatcher {
    enum_: Ident,
    ready_queue: Ident,
}

struct Task {
    alias: Ident,
    free_queue: Ident,
    inputs: Ident,
    spawn_fn: Ident,

    #[cfg(feature = "timer-queue")]
    scheduleds: Ident,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            #[cfg(feature = "timer-queue")]
            baseline: mk_ident(None),
            dispatchers: HashMap::new(),
            idle: mk_ident(Some("idle")),
            init: mk_ident(Some("init")),
            priority: mk_ident(None),
            statics: Aliases::new(),
            resources: HashMap::new(),
            schedule_enum: mk_ident(None),
            schedule_fn: Aliases::new(),
            tasks: HashMap::new(),
            timer_queue: mk_ident(None),
        }
    }
}

struct Resources {
    alias: Ident,
    decl: proc_macro2::TokenStream,
}

pub fn app(app: &App, analysis: &Analysis) -> TokenStream {
    let mut ctxt = Context::default();

    let resources = resources(&mut ctxt, &app, analysis);

    let tasks = tasks(&mut ctxt, &app, analysis);

    let (dispatchers_data, dispatchers) = dispatchers(&mut ctxt, &app, analysis);

    let init_fn = init(&mut ctxt, &app, analysis);
    let init_arg = if cfg!(feature = "timer-queue") {
        quote!(rtfm::Peripherals {
            CBP: p.CBP,
            CPUID: p.CPUID,
            DCB: &mut p.DCB,
            FPB: p.FPB,
            FPU: p.FPU,
            ITM: p.ITM,
            MPU: p.MPU,
            SCB: &mut p.SCB,
            TPIU: p.TPIU,
        })
    } else {
        quote!(rtfm::Peripherals {
            CBP: p.CBP,
            CPUID: p.CPUID,
            DCB: p.DCB,
            DWT: p.DWT,
            FPB: p.FPB,
            FPU: p.FPU,
            ITM: p.ITM,
            MPU: p.MPU,
            SCB: &mut p.SCB,
            SYST: p.SYST,
            TPIU: p.TPIU,
        })
    };

    let post_init = post_init(&ctxt, &app, analysis);

    let (idle_fn, idle_expr) = idle(&mut ctxt, &app, analysis);

    let exceptions = exceptions(&mut ctxt, app, analysis);

    let (root_interrupts, scoped_interrupts) = interrupts(&mut ctxt, app, analysis);

    let spawn = spawn(&mut ctxt, app, analysis);

    let schedule = match () {
        #[cfg(feature = "timer-queue")]
        () => schedule(&ctxt, app),
        #[cfg(not(feature = "timer-queue"))]
        () => quote!(),
    };

    let timer_queue = timer_queue(&ctxt, app, analysis);

    let pre_init = pre_init(&ctxt, &app, analysis);

    let assertions = assertions(app, analysis);

    let main = mk_ident(None);
    let init = &ctxt.init;
    quote!(
        #resources

        #spawn

        #timer_queue

        #schedule

        #dispatchers_data

        #(#exceptions)*

        #root_interrupts

        const APP: () = {
            #scoped_interrupts

            #(#dispatchers)*
        };

        #(#tasks)*

        #init_fn

        #idle_fn

        #[export_name = "main"]
        #[allow(unsafe_code)]
        #[doc(hidden)]
        unsafe fn #main() -> ! {
            #assertions

            rtfm::export::interrupt::disable();

            #pre_init

            #init(#init_arg);

            #post_init

            rtfm::export::interrupt::enable();

            #idle_expr
        }
    )
    .into()
}

fn resources(ctxt: &mut Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let mut items = vec![];
    let mut module = vec![];
    for (name, res) in &app.resources {
        let cfgs = &res.cfgs;
        let attrs = &res.attrs;
        let mut_ = &res.mutability;
        let ty = &res.ty;
        let expr = &res.expr;

        if res.singleton {
            items.push(quote!(
                #(#attrs)*
                pub static #mut_ #name: #ty = #expr;
            ));

            let alias = mk_ident(None);
            if let Some(Ownership::Shared { ceiling }) = analysis.ownerships.get(name) {
                items.push(mk_resource(
                    ctxt,
                    cfgs,
                    name,
                    quote!(#name),
                    *ceiling,
                    quote!(&mut <#name as owned_singleton::Singleton>::new()),
                    app,
                    Some(&mut module),
                ))
            }

            ctxt.statics.insert(name.clone(), alias);
        } else {
            let alias = mk_ident(None);
            let symbol = format!("{}::{}", name, alias);

            items.push(
                expr.as_ref()
                    .map(|expr| {
                        quote!(
                            #(#attrs)*
                            #(#cfgs)*
                            #[doc = #symbol]
                            static mut #alias: #ty = #expr;
                        )
                    })
                    .unwrap_or_else(|| {
                        quote!(
                            #(#attrs)*
                            #(#cfgs)*
                            #[doc = #symbol]
                            static mut #alias: rtfm::export::MaybeUninit<#ty> =
                                rtfm::export::MaybeUninit::uninitialized();
                        )
                    }),
            );

            if let Some(Ownership::Shared { ceiling }) = analysis.ownerships.get(name) {
                if res.mutability.is_some() {
                    let ptr = if res.expr.is_none() {
                        quote!(unsafe { #alias.get_mut() })
                    } else {
                        quote!(unsafe { &mut #alias })
                    };

                    items.push(mk_resource(
                        ctxt,
                        cfgs,
                        name,
                        quote!(#ty),
                        *ceiling,
                        ptr,
                        app,
                        Some(&mut module),
                    ));
                }
            }

            ctxt.statics.insert(name.clone(), alias);
        }
    }

    if !module.is_empty() {
        items.push(quote!(
            /// Resource proxies
            pub mod resources {
                #(#module)*
            }
        ));
    }

    quote!(#(#items)*)
}

fn init(ctxt: &mut Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let attrs = &app.init.attrs;
    let locals = mk_locals(&app.init.statics, true);
    let stmts = &app.init.stmts;
    let assigns = app
        .init
        .assigns
        .iter()
        .map(|assign| {
            let attrs = &assign.attrs;
            if app
                .resources
                .get(&assign.left)
                .map(|r| r.expr.is_none())
                .unwrap_or(false)
            {
                let alias = &ctxt.statics[&assign.left];
                let expr = &assign.right;
                quote!(
                    #(#attrs)*
                    unsafe { #alias.set(#expr); }
                )
            } else {
                let left = &assign.left;
                let right = &assign.right;
                quote!(
                    #(#attrs)*
                    #left = #right;
                )
            }
        })
        .collect::<Vec<_>>();

    let prelude = prelude(
        ctxt,
        Kind::Init,
        &app.init.args.resources,
        &app.init.args.spawn,
        &app.init.args.schedule,
        app,
        255,
        analysis,
    );

    let module = module(
        ctxt,
        Kind::Init,
        !app.init.args.schedule.is_empty(),
        !app.init.args.spawn.is_empty(),
        app,
    );

    #[cfg(feature = "timer-queue")]
    let baseline = &ctxt.baseline;
    let baseline_let = match () {
        #[cfg(feature = "timer-queue")]
        () => quote!(let #baseline = rtfm::Instant::artificial(0);),

        #[cfg(not(feature = "timer-queue"))]
        () => quote!(),
    };

    let start_let = match () {
        #[cfg(feature = "timer-queue")]
        () => quote!(
            #[allow(unused_variables)]
            let start = #baseline;
        ),
        #[cfg(not(feature = "timer-queue"))]
        () => quote!(),
    };

    let unsafety = &app.init.unsafety;
    let device = &app.args.device;
    let init = &ctxt.init;
    quote!(
        #module

        #(#attrs)*
        #unsafety fn #init(mut core: rtfm::Peripherals) {
            #(#locals)*

            #baseline_let

            #prelude

            let mut device = unsafe { #device::Peripherals::steal() };

            #start_let

            #(#stmts)*

            #(#assigns)*
        }
    )
}

fn post_init(ctxt: &Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let mut exprs = vec![];

    // TODO turn the assertions that check that the priority is not larger than what's supported by
    // the device into compile errors
    let device = &app.args.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);
    for (name, exception) in &app.exceptions {
        let priority = exception.args.priority;
        exprs.push(quote!(assert!(#priority <= (1 << #nvic_prio_bits))));
        exprs.push(quote!(p.SCB.set_priority(
            rtfm::export::SystemHandler::#name,
            ((1 << #nvic_prio_bits) - #priority) << (8 - #nvic_prio_bits),
        )));
    }

    if !analysis.timer_queue.tasks.is_empty() {
        let priority = analysis.timer_queue.priority;
        exprs.push(quote!(assert!(#priority <= (1 << #nvic_prio_bits))));
        exprs.push(quote!(p.SCB.set_priority(
            rtfm::export::SystemHandler::SysTick,
            ((1 << #nvic_prio_bits) - #priority) << (8 - #nvic_prio_bits),
        )));
    }

    if app.idle.is_none() {
        // Set SLEEPONEXIT bit to enter sleep mode when returning from ISR
        exprs.push(quote!(p.SCB.scr.modify(|r| r | 1 << 1)));
    }

    // Enable and start the system timer
    if !analysis.timer_queue.tasks.is_empty() {
        let tq = &ctxt.timer_queue;
        exprs.push(quote!(#tq.get_mut().syst.set_clock_source(rtfm::export::SystClkSource::Core)));
        exprs.push(quote!(#tq.get_mut().syst.enable_counter()));
    }

    // Enable cycle counter
    if cfg!(feature = "timer-queue") {
        exprs.push(quote!(p.DCB.enable_trace()));
        exprs.push(quote!(p.DWT.enable_cycle_counter()));
    }

    quote!(#(#exprs;)*)
}

/// This function creates creates a module for `init` / `idle` / a `task` (see `kind` argument)
fn module(
    ctxt: &mut Context,
    kind: Kind,
    schedule: bool,
    spawn: bool,
    app: &App,
) -> proc_macro2::TokenStream {
    let mut items = vec![];
    let mut fields = vec![];

    let name = kind.ident();
    let priority = &ctxt.priority;
    let device = &app.args.device;

    let mut lt = None;
    match kind {
        Kind::Init => {
            if cfg!(feature = "timer-queue") {
                fields.push(quote!(
                    /// System start time = `Instant(0 /* cycles */)`
                    pub start: rtfm::Instant,
                ));
            }

            fields.push(quote!(
                /// Core (Cortex-M) peripherals
                pub core: rtfm::Peripherals<'a>,
                /// Device specific peripherals
                pub device: #device::Peripherals,
            ));
            lt = Some(quote!('a));
        }
        Kind::Idle => {}
        Kind::Exception(_) | Kind::Interrupt(_) => {
            if cfg!(feature = "timer-queue") {
                fields.push(quote!(
                    /// Time at which this handler started executing
                    pub start: rtfm::Instant,
                ));
            }
        }
        Kind::Task(_) => {
            if cfg!(feature = "timer-queue") {
                fields.push(quote!(
                    /// The time at which this task was scheduled to run
                    pub scheduled: rtfm::Instant,
                ));
            }
        }
    }

    if schedule {
        lt = Some(quote!('a));

        fields.push(quote!(
            /// Tasks that can be scheduled from this context
            pub schedule: Schedule<'a>,
        ));

        items.push(quote!(
            /// Tasks that can be scheduled from this context
            #[derive(Clone, Copy)]
            pub struct Schedule<'a> {
                #[doc(hidden)]
                pub #priority: &'a core::cell::Cell<u8>,
            }
        ));
    }

    if spawn {
        lt = Some(quote!('a));

        fields.push(quote!(
            /// Tasks that can be spawned from this context
            pub spawn: Spawn<'a>,
        ));

        if kind.is_idle() {
            items.push(quote!(
                /// Tasks that can be spawned from this context
                #[derive(Clone, Copy)]
                pub struct Spawn<'a> {
                    #[doc(hidden)]
                    pub #priority: &'a core::cell::Cell<u8>,
                }
            ));
        } else {
            let baseline_field = match () {
                #[cfg(feature = "timer-queue")]
                () => {
                    let baseline = &ctxt.baseline;
                    quote!(
                        #[doc(hidden)]
                        pub #baseline: rtfm::Instant,
                    )
                }
                #[cfg(not(feature = "timer-queue"))]
                () => quote!(),
            };

            items.push(quote!(
                /// Tasks that can be spawned from this context
                #[derive(Clone, Copy)]
                pub struct Spawn<'a> {
                    #baseline_field
                    #[doc(hidden)]
                    pub #priority: &'a core::cell::Cell<u8>,
                }
            ));
        }
    }

    let mut root = None;
    if let Some(resources) = ctxt.resources.get(&kind) {
        lt = Some(quote!('a));

        root = Some(resources.decl.clone());

        let alias = &resources.alias;
        items.push(quote!(
            #[doc(inline)]
            pub use super::#alias as Resources;
        ));

        fields.push(quote!(
            /// Resources available in this context
            pub resources: Resources<'a>,
        ));
    };

    let doc = match kind {
        Kind::Exception(_) => "Exception handler",
        Kind::Idle => "Idle loop",
        Kind::Init => "Initialization function",
        Kind::Interrupt(_) => "Interrupt handler",
        Kind::Task(_) => "Software task",
    };

    quote!(
        #root

        #[doc = #doc]
        pub mod #name {
            /// Variables injected into this context by the `app` attribute
            pub struct Context<#lt> {
                #(#fields)*
            }

            #(#items)*
        }
    )
}

/// The prelude injects `resources`, `spawn`, `schedule` and `start` / `scheduled` (all values) into
/// a function scope
fn prelude(
    ctxt: &mut Context,
    kind: Kind,
    resources: &Idents,
    spawn: &Idents,
    schedule: &Idents,
    app: &App,
    logical_prio: u8,
    analysis: &Analysis,
) -> proc_macro2::TokenStream {
    let mut items = vec![];

    let lt = if kind.runs_once() {
        quote!('static)
    } else {
        quote!('a)
    };

    let module = kind.ident();

    let priority = &ctxt.priority;
    if !resources.is_empty() {
        let mut defs = vec![];
        let mut exprs = vec![];

        // NOTE This field is just to avoid unused type parameter errors around `'a`
        defs.push(quote!(#[allow(dead_code)] #priority: &'a core::cell::Cell<u8>));
        exprs.push(parse_quote!(#priority));

        let mut may_call_lock = false;
        let mut needs_unsafe = false;
        for name in resources {
            let res = &app.resources[name];
            let cfgs = &res.cfgs;

            let initialized = res.expr.is_some();
            let singleton = res.singleton;
            let mut_ = res.mutability;
            let ty = &res.ty;

            if kind.is_init() {
                let mut force_mut = false;
                if !analysis.ownerships.contains_key(name) {
                    // owned by Init
                    if singleton {
                        needs_unsafe = true;
                        defs.push(quote!(
                            #(#cfgs)*
                            pub #name: #name
                        ));
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: <#name as owned_singleton::Singleton>::new()
                        ));
                        continue;
                    } else {
                        defs.push(quote!(
                            #(#cfgs)*
                            pub #name: &'static #mut_ #ty
                        ));
                    }
                } else {
                    // owned by someone else
                    if singleton {
                        needs_unsafe = true;
                        defs.push(quote!(
                            #(#cfgs)*
                            pub #name: &'a mut #name
                        ));
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: &mut <#name as owned_singleton::Singleton>::new()
                        ));
                        continue;
                    } else {
                        force_mut = true;
                        defs.push(quote!(
                            #(#cfgs)*
                            pub #name: &'a mut #ty
                        ));
                    }
                }

                let alias = &ctxt.statics[name];
                // Resources assigned to init are always const initialized
                needs_unsafe = true;
                if force_mut {
                    exprs.push(quote!(
                        #(#cfgs)*
                        #name: &mut #alias
                    ));
                } else {
                    exprs.push(quote!(
                        #(#cfgs)*
                        #name: &#mut_ #alias
                    ));
                }
            } else {
                let ownership = &analysis.ownerships[name];
                let mut exclusive = false;

                if ownership.needs_lock(logical_prio) {
                    may_call_lock = true;
                    if singleton {
                        if mut_.is_none() {
                            needs_unsafe = true;
                            defs.push(quote!(
                                #(#cfgs)*
                                pub #name: &'a #name
                            ));
                            exprs.push(quote!(
                                #(#cfgs)*
                                #name: &<#name as owned_singleton::Singleton>::new()
                            ));
                            continue;
                        } else {
                            // Generate a resource proxy
                            defs.push(quote!(
                                #(#cfgs)*
                                pub #name: resources::#name<'a>
                            ));
                            exprs.push(quote!(
                                #(#cfgs)*
                                #name: resources::#name { #priority }
                            ));
                            continue;
                        }
                    } else {
                        if mut_.is_none() {
                            defs.push(quote!(
                                #(#cfgs)*
                                pub #name: &'a #ty
                            ));
                        } else {
                            // Generate a resource proxy
                            defs.push(quote!(
                                #(#cfgs)*
                                pub #name: resources::#name<'a>
                            ));
                            exprs.push(quote!(
                                #(#cfgs)*
                                #name: resources::#name { #priority }
                            ));
                            continue;
                        }
                    }
                } else {
                    if singleton {
                        if kind.runs_once() {
                            needs_unsafe = true;
                            defs.push(quote!(
                                #(#cfgs)*
                                pub #name: #name
                            ));
                            exprs.push(quote!(
                                #(#cfgs)*
                                #name: <#name as owned_singleton::Singleton>::new()
                            ));
                        } else {
                            needs_unsafe = true;
                            if ownership.is_owned() || mut_.is_none() {
                                defs.push(quote!(
                                    #(#cfgs)*
                                    pub #name: &'a #mut_ #name
                                ));
                                let alias = mk_ident(None);
                                items.push(quote!(
                                    #(#cfgs)*
                                    let #mut_ #alias = unsafe {
                                        <#name as owned_singleton::Singleton>::new()
                                    };
                                ));
                                exprs.push(quote!(
                                    #(#cfgs)*
                                    #name: &#mut_ #alias
                                ));
                            } else {
                                may_call_lock = true;
                                defs.push(quote!(
                                    #(#cfgs)*
                                    pub #name: rtfm::Exclusive<'a, #name>
                                ));
                                let alias = mk_ident(None);
                                items.push(quote!(
                                    #(#cfgs)*
                                    let #mut_ #alias = unsafe {
                                        <#name as owned_singleton::Singleton>::new()
                                    };
                                ));
                                exprs.push(quote!(
                                    #(#cfgs)*
                                    #name: rtfm::Exclusive(&mut #alias)
                                ));
                            }
                        }
                        continue;
                    } else {
                        if ownership.is_owned() || mut_.is_none() {
                            defs.push(quote!(
                                #(#cfgs)*
                                pub #name: &#lt #mut_ #ty
                            ));
                        } else {
                            exclusive = true;
                            may_call_lock = true;
                            defs.push(quote!(
                                #(#cfgs)*
                                pub #name: rtfm::Exclusive<#lt, #ty>
                            ));
                        }
                    }
                }

                let alias = &ctxt.statics[name];
                needs_unsafe = true;
                if initialized {
                    if exclusive {
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: rtfm::Exclusive(&mut #alias)
                        ));
                    } else {
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: &#mut_ #alias
                        ));
                    }
                } else {
                    let method = if mut_.is_some() {
                        quote!(get_mut)
                    } else {
                        quote!(get_ref)
                    };

                    if exclusive {
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: rtfm::Exclusive(#alias.#method())
                        ));
                    } else {
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: #alias.#method()
                        ));
                    }
                }
            }
        }

        let alias = mk_ident(None);
        let unsafety = if needs_unsafe {
            Some(quote!(unsafe))
        } else {
            None
        };

        let defs = &defs;
        let doc = format!("`{}::Resources`", kind.ident().to_string());
        let decl = quote!(
            #[doc = #doc]
            #[allow(non_snake_case)]
            pub struct #alias<'a> { #(#defs,)* }
        );
        items.push(quote!(
            #[allow(unused_variables)]
            #[allow(unsafe_code)]
            #[allow(unused_mut)]
            let mut resources = #unsafety { #alias { #(#exprs,)* } };
        ));

        ctxt.resources
            .insert(kind.clone(), Resources { alias, decl });

        if may_call_lock {
            items.push(quote!(
                use rtfm::Mutex;
            ));
        }
    }

    if !spawn.is_empty() {
        if kind.is_idle() {
            items.push(quote!(
                #[allow(unused_variables)]
                let spawn = #module::Spawn { #priority };
            ));
        } else {
            let baseline_expr = match () {
                #[cfg(feature = "timer-queue")]
                () => {
                    let baseline = &ctxt.baseline;
                    quote!(#baseline)
                }
                #[cfg(not(feature = "timer-queue"))]
                () => quote!(),
            };
            items.push(quote!(
                #[allow(unused_variables)]
                let spawn = #module::Spawn { #priority, #baseline_expr };
            ));
        }
    }

    if !schedule.is_empty() {
        // Populate `schedule_fn`
        for task in schedule {
            if ctxt.schedule_fn.contains_key(task) {
                continue;
            }

            ctxt.schedule_fn.insert(task.clone(), mk_ident(None));
        }

        items.push(quote!(
            #[allow(unused_imports)]
            use rtfm::U32Ext;

            #[allow(unused_variables)]
            let schedule = #module::Schedule { #priority };
        ));
    }

    if items.is_empty() {
        quote!()
    } else {
        quote!(
            let ref #priority = core::cell::Cell::new(#logical_prio);

            #(#items)*
        )
    }
}

fn idle(
    ctxt: &mut Context,
    app: &App,
    analysis: &Analysis,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if let Some(idle) = app.idle.as_ref() {
        let attrs = &idle.attrs;
        let locals = mk_locals(&idle.statics, true);
        let stmts = &idle.stmts;

        let prelude = prelude(
            ctxt,
            Kind::Idle,
            &idle.args.resources,
            &idle.args.spawn,
            &idle.args.schedule,
            app,
            0,
            analysis,
        );

        let module = module(
            ctxt,
            Kind::Idle,
            !idle.args.schedule.is_empty(),
            !idle.args.spawn.is_empty(),
            app,
        );

        let unsafety = &idle.unsafety;
        let idle = &ctxt.idle;

        (
            quote!(
                #module

                #(#attrs)*
                #unsafety fn #idle() -> ! {
                    #(#locals)*

                    #prelude

                    #(#stmts)*
                }
            ),
            quote!(#idle()),
        )
    } else {
        (
            quote!(),
            quote!(loop {
                rtfm::export::wfi();
            }),
        )
    }
}

fn exceptions(ctxt: &mut Context, app: &App, analysis: &Analysis) -> Vec<proc_macro2::TokenStream> {
    app.exceptions
        .iter()
        .map(|(ident, exception)| {
            let attrs = &exception.attrs;
            let stmts = &exception.stmts;

            let prelude = prelude(
                ctxt,
                Kind::Exception(ident.clone()),
                &exception.args.resources,
                &exception.args.spawn,
                &exception.args.schedule,
                app,
                exception.args.priority,
                analysis,
            );

            let module = module(
                ctxt,
                Kind::Exception(ident.clone()),
                !exception.args.schedule.is_empty(),
                !exception.args.spawn.is_empty(),
                app,
            );

            #[cfg(feature = "timer-queue")]
            let baseline = &ctxt.baseline;
            let baseline_let = match () {
                #[cfg(feature = "timer-queue")]
                () => quote!(let #baseline = rtfm::Instant::now();),
                #[cfg(not(feature = "timer-queue"))]
                () => quote!(),
            };

            let start_let = match () {
                #[cfg(feature = "timer-queue")]
                () => quote!(
                    #[allow(unused_variables)]
                    let start = #baseline;
                ),
                #[cfg(not(feature = "timer-queue"))]
                () => quote!(),
            };

            let locals = mk_locals(&exception.statics, false);
            let symbol = ident.to_string();
            let alias = mk_ident(None);
            let unsafety = &exception.unsafety;
            quote!(
                #module

                #[doc(hidden)]
                #[export_name = #symbol]
                #(#attrs)*
                #unsafety fn #alias() {
                    #(#locals)*

                    #baseline_let

                    #prelude

                    #start_let

                    rtfm::export::run(move || {
                        #(#stmts)*
                    })
                }
            )
        })
        .collect()
}

fn interrupts(
    ctxt: &mut Context,
    app: &App,
    analysis: &Analysis,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut root = vec![];
    let mut scoped = vec![];

    let device = &app.args.device;
    for (ident, interrupt) in &app.interrupts {
        let attrs = &interrupt.attrs;
        let stmts = &interrupt.stmts;

        let prelude = prelude(
            ctxt,
            Kind::Interrupt(ident.clone()),
            &interrupt.args.resources,
            &interrupt.args.spawn,
            &interrupt.args.schedule,
            app,
            interrupt.args.priority,
            analysis,
        );

        root.push(module(
            ctxt,
            Kind::Interrupt(ident.clone()),
            !interrupt.args.schedule.is_empty(),
            !interrupt.args.spawn.is_empty(),
            app,
        ));

        #[cfg(feature = "timer-queue")]
        let baseline = &ctxt.baseline;
        let baseline_let = match () {
            #[cfg(feature = "timer-queue")]
            () => quote!(let #baseline = rtfm::Instant::now();),
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let start_let = match () {
            #[cfg(feature = "timer-queue")]
            () => quote!(
                #[allow(unused_variables)]
                let start = #baseline;
            ),
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let locals = mk_locals(&interrupt.statics, false);
        let alias = mk_ident(None);
        let symbol = ident.to_string();
        let unsafety = &interrupt.unsafety;
        scoped.push(quote!(
            #(#attrs)*
            #[export_name = #symbol]
            #unsafety fn #alias() {
                // check that this interrupt exists
                let _ = #device::interrupt::#ident;

                #(#locals)*

                #baseline_let

                #prelude

                #start_let

                rtfm::export::run(move || {
                    #(#stmts)*
                })
            }
        ));
    }

    (quote!(#(#root)*), quote!(#(#scoped)*))
}

fn tasks(ctxt: &mut Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let mut items = vec![];

    // first pass to generate buffers (statics and resources) and spawn aliases
    for (name, task) in &app.tasks {
        #[cfg(feature = "timer-queue")]
        let scheduleds_alias = mk_ident(None);
        let free_alias = mk_ident(None);
        let inputs_alias = mk_ident(None);
        let task_alias = mk_ident(Some(&name.to_string()));

        let inputs = &task.inputs;

        let ty = tuple_ty(inputs);

        let capacity_lit = mk_capacity_literal(analysis.capacities[name]);
        let capacity_ty = mk_typenum_capacity(analysis.capacities[name], true);

        let resource = mk_resource(
            ctxt,
            &[],
            &free_alias,
            quote!(rtfm::export::FreeQueue<#capacity_ty>),
            *analysis.free_queues.get(name).unwrap_or(&0),
            quote!(#free_alias.get_mut()),
            app,
            None,
        );

        let scheduleds_static = match () {
            #[cfg(feature = "timer-queue")]
            () => {
                let scheduleds_symbol = format!("{}::SCHEDULED_TIMES::{}", name, scheduleds_alias);

                quote!(
                    #[doc = #scheduleds_symbol]
                    static mut #scheduleds_alias:
                    rtfm::export::MaybeUninit<[rtfm::Instant; #capacity_lit]> =
                        rtfm::export::MaybeUninit::uninitialized();
                )
            }
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let inputs_symbol = format!("{}::INPUTS::{}", name, inputs_alias);
        let free_symbol = format!("{}::FREE_QUEUE::{}", name, free_alias);
        items.push(quote!(
            // FIXME(MaybeUninit) MaybeUninit won't be necessary when core::mem::MaybeUninit
            // stabilizes because heapless constructors will work in const context
            #[doc = #free_symbol]
            static mut #free_alias: rtfm::export::MaybeUninit<
                    rtfm::export::FreeQueue<#capacity_ty>
                > = rtfm::export::MaybeUninit::uninitialized();

            #resource

            #[doc = #inputs_symbol]
            static mut #inputs_alias: rtfm::export::MaybeUninit<[#ty; #capacity_lit]> =
                rtfm::export::MaybeUninit::uninitialized();

            #scheduleds_static
        ));

        ctxt.tasks.insert(
            name.clone(),
            Task {
                alias: task_alias,
                free_queue: free_alias,
                inputs: inputs_alias,
                spawn_fn: mk_ident(None),

                #[cfg(feature = "timer-queue")]
                scheduleds: scheduleds_alias,
            },
        );
    }

    // second pass to generate the actual task function
    for (name, task) in &app.tasks {
        let inputs = &task.inputs;
        let locals = mk_locals(&task.statics, false);
        let stmts = &task.stmts;
        let unsafety = &task.unsafety;

        let scheduled_let = match () {
            #[cfg(feature = "timer-queue")]
            () => {
                let baseline = &ctxt.baseline;
                quote!(let scheduled = #baseline;)
            }
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let prelude = prelude(
            ctxt,
            Kind::Task(name.clone()),
            &task.args.resources,
            &task.args.spawn,
            &task.args.schedule,
            app,
            task.args.priority,
            analysis,
        );

        items.push(module(
            ctxt,
            Kind::Task(name.clone()),
            !task.args.schedule.is_empty(),
            !task.args.spawn.is_empty(),
            app,
        ));

        let attrs = &task.attrs;
        let cfgs = &task.cfgs;
        let task_alias = &ctxt.tasks[name].alias;
        let baseline_arg = match () {
            #[cfg(feature = "timer-queue")]
            () => {
                let baseline = &ctxt.baseline;
                quote!(#baseline: rtfm::Instant,)
            }
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };
        items.push(quote!(
            #(#attrs)*
            #(#cfgs)*
            #unsafety fn #task_alias(#baseline_arg #(#inputs,)*) {
                #(#locals)*

                #prelude

                #scheduled_let

                #(#stmts)*
            }
        ));
    }

    quote!(#(#items)*)
}

fn dispatchers(
    ctxt: &mut Context,
    app: &App,
    analysis: &Analysis,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut data = vec![];
    let mut dispatchers = vec![];

    let device = &app.args.device;
    for (level, dispatcher) in &analysis.dispatchers {
        let ready_alias = mk_ident(None);
        let enum_alias = mk_ident(None);
        let capacity = mk_typenum_capacity(dispatcher.capacity, true);

        let variants = dispatcher
            .tasks
            .iter()
            .map(|task| {
                let task_ = &app.tasks[task];
                let cfgs = &task_.cfgs;

                quote!(
                    #(#cfgs)*
                    #task
                )
            })
            .collect::<Vec<_>>();
        let symbol = format!("P{}::READY_QUEUE::{}", level, ready_alias);
        let e = quote!(rtfm::export);
        let ty = quote!(#e::ReadyQueue<#enum_alias, #capacity>);
        let ceiling = *analysis.ready_queues.get(&level).unwrap_or(&0);
        let resource = mk_resource(
            ctxt,
            &[],
            &ready_alias,
            ty.clone(),
            ceiling,
            quote!(#ready_alias.get_mut()),
            app,
            None,
        );
        data.push(quote!(
            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            enum #enum_alias { #(#variants,)* }

            #[doc = #symbol]
            static mut #ready_alias: #e::MaybeUninit<#ty> = #e::MaybeUninit::uninitialized();

            #resource
        ));

        let arms = dispatcher
            .tasks
            .iter()
            .map(|task| {
                let task_ = &ctxt.tasks[task];
                let inputs = &task_.inputs;
                let free = &task_.free_queue;
                let alias = &task_.alias;

                let task__ = &app.tasks[task];
                let pats = tuple_pat(&task__.inputs);
                let cfgs = &task__.cfgs;

                let baseline_let;
                let call;
                match () {
                    #[cfg(feature = "timer-queue")]
                    () => {
                        let scheduleds = &task_.scheduleds;
                        baseline_let = quote!(
                            let baseline =
                                ptr::read(#scheduleds.get_ref().get_unchecked(usize::from(index)));
                        );
                        call = quote!(#alias(baseline, #pats));
                    }
                    #[cfg(not(feature = "timer-queue"))]
                    () => {
                        baseline_let = quote!();
                        call = quote!(#alias(#pats));
                    }
                };

                quote!(
                    #(#cfgs)*
                    #enum_alias::#task => {
                        #baseline_let
                        let input = ptr::read(#inputs.get_ref().get_unchecked(usize::from(index)));
                        #free.get_mut().split().0.enqueue_unchecked(index);
                        let (#pats) = input;
                        #call
                    }
                )
            })
            .collect::<Vec<_>>();

        let attrs = &dispatcher.attrs;
        let interrupt = &dispatcher.interrupt;
        let symbol = interrupt.to_string();
        let alias = mk_ident(None);
        dispatchers.push(quote!(
            #(#attrs)*
            #[export_name = #symbol]
            unsafe fn #alias() {
                use core::ptr;

                // check that this interrupt exists
                let _ = #device::interrupt::#interrupt;

                rtfm::export::run(|| {
                    while let Some((task, index)) = #ready_alias.get_mut().split().1.dequeue() {
                        match task {
                            #(#arms)*
                        }
                    }
                });
            }
        ));

        ctxt.dispatchers.insert(
            *level,
            Dispatcher {
                ready_queue: ready_alias,
                enum_: enum_alias,
            },
        );
    }

    (quote!(#(#data)*), quote!(#(#dispatchers)*))
}

fn spawn(ctxt: &Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let mut items = vec![];

    // Generate `spawn` functions
    let device = &app.args.device;
    let priority = &ctxt.priority;
    #[cfg(feature = "timer-queue")]
    let baseline = &ctxt.baseline;
    for (name, task) in &ctxt.tasks {
        let alias = &task.spawn_fn;
        let task_ = &app.tasks[name];
        let cfgs = &task_.cfgs;
        let free = &task.free_queue;
        let level = task_.args.priority;
        let dispatcher = &ctxt.dispatchers[&level];
        let ready = &dispatcher.ready_queue;
        let enum_ = &dispatcher.enum_;
        let dispatcher = &analysis.dispatchers[&level].interrupt;
        let inputs = &task.inputs;
        let args = &task_.inputs;
        let ty = tuple_ty(args);
        let pats = tuple_pat(args);

        let scheduleds_write = match () {
            #[cfg(feature = "timer-queue")]
            () => {
                let scheduleds = &ctxt.tasks[name].scheduleds;
                quote!(
                    ptr::write(
                        #scheduleds.get_mut().get_unchecked_mut(usize::from(index)),
                        #baseline,
                    );
                )
            }
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let baseline_arg = match () {
            #[cfg(feature = "timer-queue")]
            () => quote!(#baseline: rtfm::Instant,),
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        items.push(quote!(
            #[inline(always)]
            #(#cfgs)*
            unsafe fn #alias(
                #baseline_arg
                #priority: &core::cell::Cell<u8>,
                #(#args,)*
            ) -> Result<(), #ty> {
                use core::ptr;

                use rtfm::Mutex;

                if let Some(index) = (#free { #priority }).lock(|f| f.split().1.dequeue()) {
                    ptr::write(#inputs.get_mut().get_unchecked_mut(usize::from(index)), (#pats));
                    #scheduleds_write

                    #ready { #priority }.lock(|rq| {
                        rq.split().0.enqueue_unchecked((#enum_::#name, index))
                    });

                    rtfm::pend(#device::Interrupt::#dispatcher);

                    Ok(())
                } else {
                    Err((#pats))
                }
            }
        ))
    }

    // Generate `spawn` structs; these call the `spawn` functions generated above
    for (name, spawn) in app.spawn_callers() {
        if spawn.is_empty() {
            continue;
        }

        #[cfg(feature = "timer-queue")]
        let is_idle = name.to_string() == "idle";

        let mut methods = vec![];
        for task in spawn {
            let task_ = &app.tasks[task];
            let alias = &ctxt.tasks[task].spawn_fn;
            let inputs = &task_.inputs;
            let cfgs = &task_.cfgs;
            let ty = tuple_ty(inputs);
            let pats = tuple_pat(inputs);

            let instant = match () {
                #[cfg(feature = "timer-queue")]
                () => {
                    if is_idle {
                        quote!(rtfm::Instant::now(),)
                    } else {
                        quote!(self.#baseline,)
                    }
                }
                #[cfg(not(feature = "timer-queue"))]
                () => quote!(),
            };
            methods.push(quote!(
                #[allow(unsafe_code)]
                #[inline]
                #(#cfgs)*
                pub fn #task(&self, #(#inputs,)*) -> Result<(), #ty> {
                    unsafe { #alias(#instant &self.#priority, #pats) }
                }
            ));
        }

        items.push(quote!(
            impl<'a> #name::Spawn<'a> {
                #(#methods)*
            }
        ));
    }

    quote!(#(#items)*)
}

#[cfg(feature = "timer-queue")]
fn schedule(ctxt: &Context, app: &App) -> proc_macro2::TokenStream {
    let mut items = vec![];

    // Generate `schedule` functions
    let priority = &ctxt.priority;
    let timer_queue = &ctxt.timer_queue;
    for (task, alias) in &ctxt.schedule_fn {
        let task_ = &ctxt.tasks[task];
        let free = &task_.free_queue;
        let enum_ = &ctxt.schedule_enum;
        let inputs = &task_.inputs;
        let scheduleds = &task_.scheduleds;
        let task__ = &app.tasks[task];
        let args = &task__.inputs;
        let cfgs = &task__.cfgs;
        let ty = tuple_ty(args);
        let pats = tuple_pat(args);

        items.push(quote!(
            #[inline(always)]
            #(#cfgs)*
            unsafe fn #alias(
                #priority: &core::cell::Cell<u8>,
                instant: rtfm::Instant,
                #(#args,)*
            ) -> Result<(), #ty> {
                use core::ptr;

                use rtfm::Mutex;

                if let Some(index) = (#free { #priority }).lock(|f| f.split().1.dequeue()) {
                    ptr::write(#inputs.get_mut().get_unchecked_mut(usize::from(index)), (#pats));
                    ptr::write(
                        #scheduleds.get_mut().get_unchecked_mut(usize::from(index)),
                        instant,
                    );

                    let nr = rtfm::export::NotReady {
                        instant,
                        index,
                        task: #enum_::#task,
                    };

                    ({#timer_queue { #priority }}).lock(|tq| tq.enqueue_unchecked(nr));

                    Ok(())
                } else {
                    Err((#pats))
                }
            }
        ))
    }

    // Generate `Schedule` structs; these call the `schedule` functions generated above
    for (name, schedule) in app.schedule_callers() {
        if schedule.is_empty() {
            continue;
        }

        debug_assert!(!schedule.is_empty());

        let mut methods = vec![];
        for task in schedule {
            let alias = &ctxt.schedule_fn[task];
            let task_ = &app.tasks[task];
            let inputs = &task_.inputs;
            let cfgs = &task_.cfgs;
            let ty = tuple_ty(inputs);
            let pats = tuple_pat(inputs);

            methods.push(quote!(
                #[inline]
                #(#cfgs)*
                pub fn #task(
                    &self,
                    instant: rtfm::Instant,
                    #(#inputs,)*
                ) -> Result<(), #ty> {
                    unsafe { #alias(&self.#priority, instant, #pats) }
                }
            ));
        }

        items.push(quote!(
            impl<'a> #name::Schedule<'a> {
                #(#methods)*
            }
        ));
    }

    quote!(#(#items)*)
}

fn timer_queue(ctxt: &Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let tasks = &analysis.timer_queue.tasks;

    if tasks.is_empty() {
        return quote!();
    }

    let mut items = vec![];

    let variants = tasks
        .iter()
        .map(|task| {
            let cfgs = &app.tasks[task].cfgs;
            quote!(
                #(#cfgs)*
                #task
            )
        })
        .collect::<Vec<_>>();
    let enum_ = &ctxt.schedule_enum;
    items.push(quote!(
        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        enum #enum_ { #(#variants,)* }
    ));

    let cap = mk_typenum_capacity(analysis.timer_queue.capacity, false);
    let tq = &ctxt.timer_queue;
    let symbol = format!("TIMER_QUEUE::{}", tq);
    items.push(quote!(
        #[doc = #symbol]
        static mut #tq:
            rtfm::export::MaybeUninit<rtfm::export::TimerQueue<#enum_, #cap>> =
                rtfm::export::MaybeUninit::uninitialized();
    ));

    items.push(mk_resource(
        ctxt,
        &[],
        tq,
        quote!(rtfm::export::TimerQueue<#enum_, #cap>),
        analysis.timer_queue.ceiling,
        quote!(#tq.get_mut()),
        app,
        None,
    ));

    let priority = &ctxt.priority;
    let device = &app.args.device;
    let arms = tasks
        .iter()
        .map(|task| {
            let task_ = &app.tasks[task];
            let level = task_.args.priority;
            let cfgs = &task_.cfgs;
            let dispatcher_ = &ctxt.dispatchers[&level];
            let tenum = &dispatcher_.enum_;
            let ready = &dispatcher_.ready_queue;
            let dispatcher = &analysis.dispatchers[&level].interrupt;

            quote!(
                #(#cfgs)*
                #enum_::#task => {
                    (#ready { #priority }).lock(|rq| {
                        rq.split().0.enqueue_unchecked((#tenum::#task, index))
                    });

                    rtfm::pend(#device::Interrupt::#dispatcher);
                }
            )
        })
        .collect::<Vec<_>>();

    let logical_prio = analysis.timer_queue.priority;
    let alias = mk_ident(None);
    items.push(quote!(
        #[export_name = "SysTick"]
        #[doc(hidden)]
        unsafe fn #alias() {
            use rtfm::Mutex;

            let ref #priority = core::cell::Cell::new(#logical_prio);

            rtfm::export::run(|| {
                rtfm::export::sys_tick(#tq { #priority }, |task, index| {
                    match task {
                        #(#arms)*
                    }
                });
            })
        }
    ));

    quote!(#(#items)*)
}

fn pre_init(ctxt: &Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let mut exprs = vec![];

    // FIXME(MaybeUninit) Because we are using a fake MaybeUninit we need to set the Option tag to
    // Some; otherwise the get_ref and get_mut could result in UB. Also heapless collections can't
    // be constructed in const context; we have to initialize them at runtime (i.e. here).

    // these are `MaybeUninit` arrays
    for task in ctxt.tasks.values() {
        let inputs = &task.inputs;
        exprs.push(quote!(#inputs.set(core::mem::uninitialized());))
    }

    #[cfg(feature = "timer-queue")]
    for task in ctxt.tasks.values() {
        let scheduleds = &task.scheduleds;
        exprs.push(quote!(#scheduleds.set(core::mem::uninitialized());))
    }

    // these are `MaybeUninit` `ReadyQueue`s
    for dispatcher in ctxt.dispatchers.values() {
        let rq = &dispatcher.ready_queue;
        exprs.push(quote!(#rq.set(rtfm::export::ReadyQueue::new_sc());))
    }

    // these are `MaybeUninit` `FreeQueue`s
    for task in ctxt.tasks.values() {
        let fq = &task.free_queue;
        exprs.push(quote!(#fq.set(rtfm::export::FreeQueue::new_sc());))
    }

    // end-of-FIXME

    // Initialize the timer queue
    if !analysis.timer_queue.tasks.is_empty() {
        let tq = &ctxt.timer_queue;
        exprs.push(quote!(#tq.set(rtfm::export::TimerQueue::new(p.SYST));));
    }

    // Populate the `FreeQueue`s
    for (name, task) in &ctxt.tasks {
        let fq = &task.free_queue;
        let capacity = analysis.capacities[name];
        exprs.push(quote!(
            for i in 0..#capacity {
                #fq.get_mut().enqueue_unchecked(i);
            }
        ))
    }

    // TODO turn the assertions that check that the priority is not larger than what's supported by
    // the device into compile errors
    let device = &app.args.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);
    for (name, interrupt) in &app.interrupts {
        let priority = interrupt.args.priority;
        exprs.push(quote!(p.NVIC.enable(#device::Interrupt::#name);));
        exprs.push(quote!(assert!(#priority <= (1 << #nvic_prio_bits));));
        exprs.push(quote!(p.NVIC.set_priority(
            #device::Interrupt::#name,
            ((1 << #nvic_prio_bits) - #priority) << (8 - #nvic_prio_bits),
        );));
    }

    for (priority, dispatcher) in &analysis.dispatchers {
        let name = &dispatcher.interrupt;
        exprs.push(quote!(p.NVIC.enable(#device::Interrupt::#name);));
        exprs.push(quote!(assert!(#priority <= (1 << #nvic_prio_bits));));
        exprs.push(quote!(p.NVIC.set_priority(
            #device::Interrupt::#name,
            ((1 << #nvic_prio_bits) - #priority) << (8 - #nvic_prio_bits),
        );));
    }

    // Set the cycle count to 0 and disable it while `init` executes
    if cfg!(feature = "timer-queue") {
        exprs.push(quote!(p.DWT.ctrl.modify(|r| r & !1);));
        exprs.push(quote!(p.DWT.cyccnt.write(0);));
    }

    quote!(
        let mut p = rtfm::export::Peripherals::steal();
        #(#exprs)*
    )
}

fn assertions(app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let mut items = vec![];

    for ty in &analysis.assert_sync {
        items.push(quote!(rtfm::export::assert_sync::<#ty>()));
    }

    for task in &analysis.tasks_assert_send {
        let ty = tuple_ty(&app.tasks[task].inputs);
        items.push(quote!(rtfm::export::assert_send::<#ty>()));
    }

    // all late resources need to be `Send`
    for ty in &analysis.resources_assert_send {
        items.push(quote!(rtfm::export::assert_send::<#ty>()));
    }

    quote!(#(#items;)*)
}

fn mk_resource(
    ctxt: &Context,
    cfgs: &[Attribute],
    struct_: &Ident,
    ty: proc_macro2::TokenStream,
    ceiling: u8,
    ptr: proc_macro2::TokenStream,
    app: &App,
    module: Option<&mut Vec<proc_macro2::TokenStream>>,
) -> proc_macro2::TokenStream {
    let priority = &ctxt.priority;
    let device = &app.args.device;

    let mut items = vec![];

    let path = if let Some(module) = module {
        let doc = format!("`{}`", ty);
        module.push(quote!(
            #[doc = #doc]
            #(#cfgs)*
            pub struct #struct_<'a> {
                #[doc(hidden)]
                pub #priority: &'a core::cell::Cell<u8>,
            }
        ));

        quote!(resources::#struct_)
    } else {
        items.push(quote!(
            #(#cfgs)*
            struct #struct_<'a> {
                #priority: &'a core::cell::Cell<u8>,
            }
        ));

        quote!(#struct_)
    };

    items.push(quote!(
        #(#cfgs)*
        impl<'a> rtfm::Mutex for #path<'a> {
            type T = #ty;

            #[inline]
            fn lock<R, F>(&mut self, f: F) -> R
            where
                F: FnOnce(&mut Self::T) -> R,
            {
                unsafe {
                    rtfm::export::claim(
                        #ptr,
                        &self.#priority,
                        #ceiling,
                        #device::NVIC_PRIO_BITS,
                        f,
                    )
                }
            }
        }
    ));

    quote!(#(#items)*)
}

fn mk_capacity_literal(capacity: u8) -> LitInt {
    LitInt::new(u64::from(capacity), IntSuffix::None, Span::call_site())
}

fn mk_typenum_capacity(capacity: u8, power_of_two: bool) -> proc_macro2::TokenStream {
    let capacity = if power_of_two {
        capacity
            .checked_next_power_of_two()
            .expect("capacity.next_power_of_two()")
    } else {
        capacity
    };

    let ident = Ident::new(&format!("U{}", capacity), Span::call_site());

    quote!(rtfm::export::consts::#ident)
}

fn mk_ident(name: Option<&str>) -> Ident {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let secs = elapsed.as_secs();
    let nanos = elapsed.subsec_nanos();

    let count = CALL_COUNT.fetch_add(1, Ordering::SeqCst) as u32;
    let mut seed: [u8; 16] = [0; 16];

    for (i, v) in seed.iter_mut().take(8).enumerate() {
        *v = ((secs >> (i * 8)) & 0xFF) as u8
    }

    for (i, v) in seed.iter_mut().skip(8).take(4).enumerate() {
        *v = ((nanos >> (i * 8)) & 0xFF) as u8
    }

    for (i, v) in seed.iter_mut().skip(12).enumerate() {
        *v = ((count >> (i * 8)) & 0xFF) as u8
    }

    let n;
    let mut s = if let Some(name) = name {
        n = 4;
        format!("{}_", name)
    } else {
        n = 16;
        String::new()
    };

    let mut rng = rand::rngs::SmallRng::from_seed(seed);
    for i in 0..n {
        if i == 0 || rng.gen() {
            s.push(('a' as u8 + rng.gen::<u8>() % 25) as char)
        } else {
            s.push(('0' as u8 + rng.gen::<u8>() % 10) as char)
        }
    }

    Ident::new(&s, Span::call_site())
}

// `once = true` means that these locals will be called from a function that will run *once*
fn mk_locals(locals: &HashMap<Ident, Static>, once: bool) -> proc_macro2::TokenStream {
    let lt = if once { Some(quote!('static)) } else { None };

    let locals = locals
        .iter()
        .map(|(name, static_)| {
            let attrs = &static_.attrs;
            let cfgs = &static_.cfgs;
            let expr = &static_.expr;
            let ident = name;
            let ty = &static_.ty;

            quote!(
                #[allow(non_snake_case)]
                #(#cfgs)*
                let #ident: &#lt mut #ty = {
                    #(#attrs)*
                    #(#cfgs)*
                    static mut #ident: #ty = #expr;

                    unsafe { &mut #ident }
                };
            )
        })
        .collect::<Vec<_>>();

    quote!(#(#locals)*)
}

fn tuple_pat(inputs: &[ArgCaptured]) -> proc_macro2::TokenStream {
    if inputs.len() == 1 {
        let pat = &inputs[0].pat;
        quote!(#pat)
    } else {
        let pats = inputs.iter().map(|i| &i.pat).collect::<Vec<_>>();

        quote!(#(#pats,)*)
    }
}

fn tuple_ty(inputs: &[ArgCaptured]) -> proc_macro2::TokenStream {
    if inputs.len() == 1 {
        let ty = &inputs[0].ty;
        quote!(#ty)
    } else {
        let tys = inputs.iter().map(|i| &i.ty).collect::<Vec<_>>();

        quote!((#(#tys,)*))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Kind {
    Exception(Ident),
    Idle,
    Init,
    Interrupt(Ident),
    Task(Ident),
}

impl Kind {
    fn ident(&self) -> Ident {
        match self {
            Kind::Init => Ident::new("init", Span::call_site()),
            Kind::Idle => Ident::new("idle", Span::call_site()),
            Kind::Task(name) | Kind::Interrupt(name) | Kind::Exception(name) => name.clone(),
        }
    }

    fn is_idle(&self) -> bool {
        *self == Kind::Idle
    }

    fn is_init(&self) -> bool {
        *self == Kind::Init
    }

    fn runs_once(&self) -> bool {
        match *self {
            Kind::Init | Kind::Idle => true,
            _ => false,
        }
    }
}
