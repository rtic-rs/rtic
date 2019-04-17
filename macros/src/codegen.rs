#![deny(warnings)]

use proc_macro::TokenStream;
use std::{
    collections::{BTreeMap, HashMap},
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
type Aliases = BTreeMap<Ident, Ident>;

struct Context {
    // Alias
    #[cfg(feature = "timer-queue")]
    baseline: Ident,
    dispatchers: BTreeMap<u8, Dispatcher>,
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
    tasks: BTreeMap<Ident, Task>,
    // Alias (`struct` / `static mut`)
    timer_queue: Ident,
    // Generator of Ident names or suffixes
    ident_gen: IdentGenerator,
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
        let mut ident_gen = IdentGenerator::new();

        Context {
            #[cfg(feature = "timer-queue")]
            baseline: ident_gen.mk_ident(None, false),
            dispatchers: BTreeMap::new(),
            idle: ident_gen.mk_ident(Some("idle"), false),
            init: ident_gen.mk_ident(Some("init"), false),
            priority: ident_gen.mk_ident(None, false),
            statics: Aliases::new(),
            resources: HashMap::new(),
            schedule_enum: ident_gen.mk_ident(None, false),
            schedule_fn: Aliases::new(),
            tasks: BTreeMap::new(),
            timer_queue: ident_gen.mk_ident(None, false),
            ident_gen,
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

    let (init_fn, has_late_resources) = init(&mut ctxt, &app, analysis);
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

    let init = &ctxt.init;
    let init_phase = if has_late_resources {
        let assigns = app
            .resources
            .iter()
            .filter_map(|(name, res)| {
                if res.expr.is_none() {
                    let alias = &ctxt.statics[name];

                    Some(quote!(#alias.write(res.#name);))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        quote!(
            let res = #init(#init_arg);
            #(#assigns)*
        )
    } else {
        quote!(#init(#init_arg);)
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

    let timer_queue = timer_queue(&mut ctxt, app, analysis);

    let pre_init = pre_init(&ctxt, &app, analysis);

    let assertions = assertions(app, analysis);

    let main = ctxt.ident_gen.mk_ident(None, false);
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

            #init_phase

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

            let alias = ctxt.ident_gen.mk_ident(None, true); // XXX is randomness required?
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
            let alias = ctxt.ident_gen.mk_ident(None, false);
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
                                rtfm::export::MaybeUninit::uninit();
                        )
                    }),
            );

            if let Some(Ownership::Shared { ceiling }) = analysis.ownerships.get(name) {
                if res.mutability.is_some() {
                    let ptr = if res.expr.is_none() {
                        quote!(unsafe { &mut *#alias.as_mut_ptr() })
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

fn init(ctxt: &mut Context, app: &App, analysis: &Analysis) -> (proc_macro2::TokenStream, bool) {
    let attrs = &app.init.attrs;
    let locals = mk_locals(&app.init.statics, true);
    let stmts = &app.init.stmts;
    // TODO remove in v0.5.x
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
                    unsafe { #alias.write(#expr); }
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

    let (late_resources, late_resources_ident, ret) = if app.init.returns_late_resources {
        // create `LateResources` struct in the root of the crate
        let ident = ctxt.ident_gen.mk_ident(None, false);

        let fields = app
            .resources
            .iter()
            .filter_map(|(name, res)| {
                if res.expr.is_none() {
                    let ty = &res.ty;
                    Some(quote!(pub #name: #ty))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let late_resources = quote!(
            #[allow(non_snake_case)]
            pub struct #ident {
                #(#fields),*
            }
        );

        (
            Some(late_resources),
            Some(ident),
            Some(quote!(-> init::LateResources)),
        )
    } else {
        (None, None, None)
    };
    let has_late_resources = late_resources.is_some();

    let module = module(
        ctxt,
        Kind::Init,
        !app.init.args.schedule.is_empty(),
        !app.init.args.spawn.is_empty(),
        app,
        late_resources_ident,
    );

    #[cfg(feature = "timer-queue")]
    let baseline = &ctxt.baseline;
    let baseline_let = match () {
        #[cfg(feature = "timer-queue")]
        () => quote!(let ref #baseline = rtfm::Instant::artificial(0);),

        #[cfg(not(feature = "timer-queue"))]
        () => quote!(),
    };

    let start_let = match () {
        #[cfg(feature = "timer-queue")]
        () => quote!(
            #[allow(unused_variables)]
            let start = *#baseline;
        ),
        #[cfg(not(feature = "timer-queue"))]
        () => quote!(),
    };

    let unsafety = &app.init.unsafety;
    let device = &app.args.device;
    let init = &ctxt.init;
    (
        quote!(
            #late_resources

            #module

            // unsafe trampoline to deter end-users from calling this non-reentrant function
            #(#attrs)*
            unsafe fn #init(core: rtfm::Peripherals) #ret {
                #[inline(always)]
                #unsafety fn init(mut core: rtfm::Peripherals) #ret {
                    #(#locals)*

                    #baseline_let

                    #prelude

                    let mut device = unsafe { #device::Peripherals::steal() };

                    #start_let

                    #(#stmts)*

                    #(#assigns)*
                }

                init(core)
            }
        ),
        has_late_resources,
    )
}

fn post_init(ctxt: &Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
    let mut exprs = vec![];

    // TODO turn the assertions that check that the priority is not larger than what's supported by
    // the device into compile errors
    let device = &app.args.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);
    for (handler, exception) in &app.exceptions {
        let name = exception.args.binds(handler);
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
        exprs.push(
            quote!((*#tq.as_mut_ptr()).syst.set_clock_source(rtfm::export::SystClkSource::Core)),
        );
        exprs.push(quote!((*#tq.as_mut_ptr()).syst.enable_counter()));
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
    late_resources: Option<Ident>,
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
                pub #priority: &'a rtfm::export::Priority,
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
                    pub #priority: &'a rtfm::export::Priority,
                }
            ));
        } else {
            let baseline_field = match () {
                #[cfg(feature = "timer-queue")]
                () => {
                    let baseline = &ctxt.baseline;
                    quote!(
                        // NOTE this field is visible so we use a shared reference to make it
                        // immutable
                        #[doc(hidden)]
                        pub #baseline: &'a rtfm::Instant,
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
                    pub #priority: &'a rtfm::export::Priority,
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

    if let Some(late_resources) = late_resources {
        items.push(quote!(
            pub use super::#late_resources as LateResources;
        ));
    }

    quote!(
        #root

        #[doc = #doc]
        #[allow(non_snake_case)]
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
        defs.push(quote!(#[allow(dead_code)] pub #priority: &'a rtfm::export::Priority));
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
                                // XXX is randomness required?
                                let alias = ctxt.ident_gen.mk_ident(None, true);
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
                                // XXX is randomness required?
                                let alias = ctxt.ident_gen.mk_ident(None, true);
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
                    let expr = if mut_.is_some() {
                        quote!(&mut *#alias.as_mut_ptr())
                    } else {
                        quote!(&*#alias.as_ptr())
                    };

                    if exclusive {
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: rtfm::Exclusive(#expr)
                        ));
                    } else {
                        exprs.push(quote!(
                            #(#cfgs)*
                            #name: #expr
                        ));
                    }
                }
            }
        }

        let alias = ctxt.ident_gen.mk_ident(None, false);
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

            ctxt.schedule_fn
                .insert(task.clone(), ctxt.ident_gen.mk_ident(None, false));
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
            let ref #priority = unsafe { rtfm::export::Priority::new(#logical_prio) };

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
            None,
        );

        let unsafety = &idle.unsafety;
        let idle = &ctxt.idle;

        (
            quote!(
                #module

                // unsafe trampoline to deter end-users from calling this non-reentrant function
                #(#attrs)*
                unsafe fn #idle() -> ! {
                    #[inline(always)]
                    #unsafety fn idle() -> ! {
                        #(#locals)*

                        #prelude

                        #(#stmts)*
                    }

                    idle()
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

            let kind = Kind::Exception(ident.clone());
            let prelude = prelude(
                ctxt,
                kind.clone(),
                &exception.args.resources,
                &exception.args.spawn,
                &exception.args.schedule,
                app,
                exception.args.priority,
                analysis,
            );

            let module = module(
                ctxt,
                kind,
                !exception.args.schedule.is_empty(),
                !exception.args.spawn.is_empty(),
                app,
                None,
            );

            #[cfg(feature = "timer-queue")]
            let baseline = &ctxt.baseline;
            let baseline_let = match () {
                #[cfg(feature = "timer-queue")]
                () => quote!(let ref #baseline = rtfm::Instant::now();),
                #[cfg(not(feature = "timer-queue"))]
                () => quote!(),
            };

            let start_let = match () {
                #[cfg(feature = "timer-queue")]
                () => quote!(
                    #[allow(unused_variables)]
                    let start = *#baseline;
                ),
                #[cfg(not(feature = "timer-queue"))]
                () => quote!(),
            };

            let locals = mk_locals(&exception.statics, false);
            let symbol = exception.args.binds(ident).to_string();
            let alias = ctxt.ident_gen.mk_ident(None, false);
            let unsafety = &exception.unsafety;
            quote!(
                #module

                // unsafe trampoline to deter end-users from calling this non-reentrant function
                #[export_name = #symbol]
                #(#attrs)*
                unsafe fn #alias() {
                    #[inline(always)]
                    #unsafety fn exception() {
                        #(#locals)*

                        #baseline_let

                        #prelude

                        #start_let

                        rtfm::export::run(move || {
                            #(#stmts)*
                        })
                    }

                    exception()
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

    for (ident, interrupt) in &app.interrupts {
        let attrs = &interrupt.attrs;
        let stmts = &interrupt.stmts;

        let kind = Kind::Interrupt(ident.clone());
        let prelude = prelude(
            ctxt,
            kind.clone(),
            &interrupt.args.resources,
            &interrupt.args.spawn,
            &interrupt.args.schedule,
            app,
            interrupt.args.priority,
            analysis,
        );

        root.push(module(
            ctxt,
            kind,
            !interrupt.args.schedule.is_empty(),
            !interrupt.args.spawn.is_empty(),
            app,
            None,
        ));

        #[cfg(feature = "timer-queue")]
        let baseline = &ctxt.baseline;
        let baseline_let = match () {
            #[cfg(feature = "timer-queue")]
            () => quote!(let ref #baseline = rtfm::Instant::now();),
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let start_let = match () {
            #[cfg(feature = "timer-queue")]
            () => quote!(
                #[allow(unused_variables)]
                let start = *#baseline;
            ),
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let locals = mk_locals(&interrupt.statics, false);
        let alias = ctxt.ident_gen.mk_ident(None, false);
        let symbol = interrupt.args.binds(ident).to_string();
        let unsafety = &interrupt.unsafety;
        scoped.push(quote!(
            // unsafe trampoline to deter end-users from calling this non-reentrant function
            #(#attrs)*
            #[export_name = #symbol]
            unsafe fn #alias() {
                #[inline(always)]
                #unsafety fn interrupt() {
                    #(#locals)*

                    #baseline_let

                    #prelude

                    #start_let

                    rtfm::export::run(move || {
                        #(#stmts)*
                    })
                }

                interrupt()
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
        let scheduleds_alias = ctxt.ident_gen.mk_ident(None, false);
        let free_alias = ctxt.ident_gen.mk_ident(None, false);
        let inputs_alias = ctxt.ident_gen.mk_ident(None, false);
        let task_alias = ctxt.ident_gen.mk_ident(Some(&name.to_string()), false);

        let inputs = &task.inputs;

        let ty = tuple_ty(inputs);

        let capacity = analysis.capacities[name];
        let capacity_lit = mk_capacity_literal(capacity);
        let capacity_ty = mk_typenum_capacity(capacity, true);

        let resource = mk_resource(
            ctxt,
            &[],
            &free_alias,
            quote!(rtfm::export::FreeQueue<#capacity_ty>),
            *analysis.free_queues.get(name).unwrap_or(&0),
            if cfg!(feature = "nightly") {
                quote!(&mut #free_alias)
            } else {
                quote!(#free_alias.get_mut())
            },
            app,
            None,
        );

        let scheduleds_static = match () {
            #[cfg(feature = "timer-queue")]
            () => {
                let scheduleds_symbol = format!("{}::SCHEDULED_TIMES::{}", name, scheduleds_alias);

                if cfg!(feature = "nightly") {
                    let inits =
                        (0..capacity).map(|_| quote!(rtfm::export::MaybeUninit::uninit()));

                    quote!(
                        #[doc = #scheduleds_symbol]
                        static mut #scheduleds_alias:
                            [rtfm::export::MaybeUninit<rtfm::Instant>; #capacity_lit] =
                                [#(#inits),*];
                    )
                } else {
                    quote!(
                        #[doc = #scheduleds_symbol]
                        static mut #scheduleds_alias:
                        rtfm::export::MaybeUninit<[rtfm::Instant; #capacity_lit]> =
                            rtfm::export::MaybeUninit::uninit();
                    )
                }
            }
            #[cfg(not(feature = "timer-queue"))]
            () => quote!(),
        };

        let inputs_symbol = format!("{}::INPUTS::{}", name, inputs_alias);
        let free_symbol = format!("{}::FREE_QUEUE::{}", name, free_alias);
        if cfg!(feature = "nightly") {
            let inits = (0..capacity).map(|_| quote!(rtfm::export::MaybeUninit::uninit()));

            items.push(quote!(
                #[doc = #free_symbol]
                static mut #free_alias: rtfm::export::FreeQueue<#capacity_ty> = unsafe {
                    rtfm::export::FreeQueue::new_sc()
                };

                #[doc = #inputs_symbol]
                static mut #inputs_alias: [rtfm::export::MaybeUninit<#ty>; #capacity_lit] =
                    [#(#inits),*];
            ));
        } else {
            items.push(quote!(
                #[doc = #free_symbol]
                static mut #free_alias: rtfm::export::MaybeUninit<
                    rtfm::export::FreeQueue<#capacity_ty>
                    > = rtfm::export::MaybeUninit::uninit();

                #[doc = #inputs_symbol]
                static mut #inputs_alias: rtfm::export::MaybeUninit<[#ty; #capacity_lit]> =
                    rtfm::export::MaybeUninit::uninit();

            ));
        }

        items.push(quote!(
            #resource

            #scheduleds_static
        ));

        ctxt.tasks.insert(
            name.clone(),
            Task {
                alias: task_alias,
                free_queue: free_alias,
                inputs: inputs_alias,
                spawn_fn: ctxt.ident_gen.mk_ident(None, false),

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
                quote!(let scheduled = *#baseline;)
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
            None,
        ));

        let attrs = &task.attrs;
        let cfgs = &task.cfgs;
        let task_alias = &ctxt.tasks[name].alias;
        let (baseline, baseline_arg) = match () {
            #[cfg(feature = "timer-queue")]
            () => {
                let baseline = &ctxt.baseline;
                (quote!(#baseline,), quote!(#baseline: &rtfm::Instant,))
            }
            #[cfg(not(feature = "timer-queue"))]
            () => (quote!(), quote!()),
        };
        let pats = tuple_pat(inputs);
        items.push(quote!(
            // unsafe trampoline to deter end-users from calling this non-reentrant function
            #(#attrs)*
            #(#cfgs)*
            unsafe fn #task_alias(#baseline_arg #(#inputs,)*) {
                #[inline(always)]
                #unsafety fn task(#baseline_arg #(#inputs,)*) {
                    #(#locals)*

                    #prelude

                    #scheduled_let

                    #(#stmts)*
                }

                task(#baseline #pats)
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
        let ready_alias = ctxt.ident_gen.mk_ident(None, false);
        let enum_alias = ctxt.ident_gen.mk_ident(None, false);
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
            if cfg!(feature = "nightly") {
                quote!(&mut #ready_alias)
            } else {
                quote!(#ready_alias.get_mut())
            },
            app,
            None,
        );

        if cfg!(feature = "nightly") {
            data.push(quote!(
                #[doc = #symbol]
                static mut #ready_alias: #ty = unsafe { #e::ReadyQueue::new_sc() };
            ));
        } else {
            data.push(quote!(
                #[doc = #symbol]
                static mut #ready_alias: #e::MaybeUninit<#ty> = #e::MaybeUninit::uninit();
            ));
        }
        data.push(quote!(
            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            enum #enum_alias { #(#variants,)* }

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
                        let scheduled = if cfg!(feature = "nightly") {
                            quote!(#scheduleds.get_unchecked(usize::from(index)).as_ptr())
                        } else {
                            quote!(#scheduleds.get_ref().get_unchecked(usize::from(index)))
                        };

                        baseline_let = quote!(
                            let baseline = ptr::read(#scheduled);
                        );
                        call = quote!(#alias(&baseline, #pats));
                    }
                    #[cfg(not(feature = "timer-queue"))]
                    () => {
                        baseline_let = quote!();
                        call = quote!(#alias(#pats));
                    }
                };

                let (free_, input) = if cfg!(feature = "nightly") {
                    (
                        quote!(#free),
                        quote!(#inputs.get_unchecked(usize::from(index)).as_ptr()),
                    )
                } else {
                    (
                        quote!(#free.get_mut()),
                        quote!(#inputs.get_ref().get_unchecked(usize::from(index))),
                    )
                };

                quote!(
                    #(#cfgs)*
                    #enum_alias::#task => {
                        #baseline_let
                        let input = ptr::read(#input);
                        #free_.split().0.enqueue_unchecked(index);
                        let (#pats) = input;
                        #call
                    }
                )
            })
            .collect::<Vec<_>>();

        let attrs = &dispatcher.attrs;
        let interrupt = &dispatcher.interrupt;
        let symbol = interrupt.to_string();
        let alias = ctxt.ident_gen.mk_ident(None, false);
        let ready_alias_ = if cfg!(feature = "nightly") {
            quote!(#ready_alias)
        } else {
            quote!(#ready_alias.get_mut())
        };
        dispatchers.push(quote!(
            #(#attrs)*
            #[export_name = #symbol]
            unsafe fn #alias() {
                use core::ptr;

                // check that this interrupt exists
                let _ = #device::interrupt::#interrupt;

                rtfm::export::run(|| {
                    while let Some((task, index)) = #ready_alias_.split().1.dequeue() {
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
                if cfg!(feature = "nightly") {
                    quote!(
                        ptr::write(
                            #scheduleds.get_unchecked_mut(usize::from(index)).as_mut_ptr(),
                            #baseline,
                        );
                    )
                } else {
                    quote!(
                        ptr::write(
                            #scheduleds.get_mut().get_unchecked_mut(usize::from(index)),
                            #baseline,
                        );
                    )
                }
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

        let input = if cfg!(feature = "nightly") {
            quote!(#inputs.get_unchecked_mut(usize::from(index)).as_mut_ptr())
        } else {
            quote!(#inputs.get_mut().get_unchecked_mut(usize::from(index)))
        };
        items.push(quote!(
            #[inline(always)]
            #(#cfgs)*
            unsafe fn #alias(
                #baseline_arg
                #priority: &rtfm::export::Priority,
                #(#args,)*
            ) -> Result<(), #ty> {
                use core::ptr;

                use rtfm::Mutex;

                if let Some(index) = (#free { #priority }).lock(|f| f.split().1.dequeue()) {
                    ptr::write(#input, (#pats));
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
                        quote!(*self.#baseline,)
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

        let input = if cfg!(feature = "nightly") {
            quote!(#inputs.get_unchecked_mut(usize::from(index)).as_mut_ptr())
        } else {
            quote!(#inputs.get_mut().get_unchecked_mut(usize::from(index)))
        };

        let scheduled = if cfg!(feature = "nightly") {
            quote!(#scheduleds.get_unchecked_mut(usize::from(index)).as_mut_ptr())
        } else {
            quote!(#scheduleds.get_mut().get_unchecked_mut(usize::from(index)))
        };
        items.push(quote!(
            #[inline(always)]
            #(#cfgs)*
            unsafe fn #alias(
                #priority: &rtfm::export::Priority,
                instant: rtfm::Instant,
                #(#args,)*
            ) -> Result<(), #ty> {
                use core::ptr;

                use rtfm::Mutex;

                if let Some(index) = (#free { #priority }).lock(|f| f.split().1.dequeue()) {
                    ptr::write(#input, (#pats));
                    ptr::write(#scheduled, instant);

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

fn timer_queue(ctxt: &mut Context, app: &App, analysis: &Analysis) -> proc_macro2::TokenStream {
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
    if cfg!(feature = "nightly") {
        items.push(quote!(
            #[doc = #symbol]
            static mut #tq: rtfm::export::MaybeUninit<rtfm::export::TimerQueue<#enum_, #cap>> =
                rtfm::export::MaybeUninit::uninit();
        ));
    } else {
        items.push(quote!(
            #[doc = #symbol]
            static mut #tq:
                rtfm::export::MaybeUninit<rtfm::export::TimerQueue<#enum_, #cap>> =
                    rtfm::export::MaybeUninit::uninit();
        ));
    }

    items.push(mk_resource(
        ctxt,
        &[],
        tq,
        quote!(rtfm::export::TimerQueue<#enum_, #cap>),
        analysis.timer_queue.ceiling,
        quote!(&mut *#tq.as_mut_ptr()),
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
    let alias = ctxt.ident_gen.mk_ident(None, false);
    items.push(quote!(
        #[export_name = "SysTick"]
        #[doc(hidden)]
        unsafe fn #alias() {
            use rtfm::Mutex;

            let ref #priority = rtfm::export::Priority::new(#logical_prio);

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

    if !cfg!(feature = "nightly") {
        // these are `MaybeUninit` arrays
        for task in ctxt.tasks.values() {
            let inputs = &task.inputs;
            exprs.push(quote!(#inputs.write(core::mem::MaybeUninit::uninit());))
        }

        #[cfg(feature = "timer-queue")]
        for task in ctxt.tasks.values() {
            let scheduleds = &task.scheduleds;
            exprs.push(quote!(#scheduleds.write(core::mem::MaybeUninit::uninit());))
        }

        // these are `MaybeUninit` `ReadyQueue`s
        for dispatcher in ctxt.dispatchers.values() {
            let rq = &dispatcher.ready_queue;
            exprs.push(quote!(#rq.write(rtfm::export::ReadyQueue::new_sc());))
        }

        // these are `MaybeUninit` `FreeQueue`s
        for task in ctxt.tasks.values() {
            let fq = &task.free_queue;
            exprs.push(quote!(#fq.write(rtfm::export::FreeQueue::new_sc());))
        }
    }

    // Initialize the timer queue
    if !analysis.timer_queue.tasks.is_empty() {
        let tq = &ctxt.timer_queue;
        exprs.push(quote!(#tq.write(rtfm::export::TimerQueue::new(p.SYST));));
    }

    // Populate the `FreeQueue`s
    for (name, task) in &ctxt.tasks {
        let fq = &task.free_queue;
        let fq_ = if cfg!(feature = "nightly") {
            quote!(#fq)
        } else {
            quote!(#fq.get_mut())
        };
        let capacity = analysis.capacities[name];
        exprs.push(quote!(
            for i in 0..#capacity {
                #fq_.enqueue_unchecked(i);
            }
        ))
    }

    let device = &app.args.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);
    for (handler, interrupt) in &app.interrupts {
        let name = interrupt.args.binds(handler);
        let priority = interrupt.args.priority;
        exprs.push(quote!(p.NVIC.enable(#device::Interrupt::#name);));
        exprs.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));
        exprs.push(quote!(p.NVIC.set_priority(
            #device::Interrupt::#name,
            ((1 << #nvic_prio_bits) - #priority) << (8 - #nvic_prio_bits),
        );));
    }

    for (priority, dispatcher) in &analysis.dispatchers {
        let name = &dispatcher.interrupt;
        exprs.push(quote!(p.NVIC.enable(#device::Interrupt::#name);));
        exprs.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));
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
            #[allow(non_camel_case_types)]
            #[doc = #doc]
            #(#cfgs)*
            pub struct #struct_<'a> {
                #[doc(hidden)]
                pub #priority: &'a rtfm::export::Priority,
            }
        ));

        quote!(resources::#struct_)
    } else {
        items.push(quote!(
            #(#cfgs)*
            struct #struct_<'a> {
                #priority: &'a rtfm::export::Priority,
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

struct IdentGenerator {
    call_count: u32,
    rng: rand::rngs::SmallRng,
}

impl IdentGenerator {
    fn new() -> IdentGenerator {
        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let secs = elapsed.as_secs();
        let nanos = elapsed.subsec_nanos();

        let mut seed: [u8; 16] = [0; 16];

        for (i, v) in seed.iter_mut().take(8).enumerate() {
            *v = ((secs >> (i * 8)) & 0xFF) as u8
        }

        for (i, v) in seed.iter_mut().skip(8).take(4).enumerate() {
            *v = ((nanos >> (i * 8)) & 0xFF) as u8
        }

        let rng = rand::rngs::SmallRng::from_seed(seed);

        IdentGenerator { call_count: 0, rng }
    }

    fn mk_ident(&mut self, name: Option<&str>, random: bool) -> Ident {
        let s = if let Some(name) = name {
            format!("{}_", name)
        } else {
            "__rtfm_internal_".to_string()
        };

        let mut s = format!("{}{}", s, self.call_count);
        self.call_count += 1;

        if random {
            s.push('_');

            for i in 0..4 {
                if i == 0 || self.rng.gen() {
                    s.push(('a' as u8 + self.rng.gen::<u8>() % 25) as char)
                } else {
                    s.push(('0' as u8 + self.rng.gen::<u8>() % 10) as char)
                }
            }
        }

        Ident::new(&s, Span::call_site())
    }
}

// `once = true` means that these locals will be called from a function that will run *once*
fn mk_locals(locals: &BTreeMap<Ident, Static>, once: bool) -> proc_macro2::TokenStream {
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
