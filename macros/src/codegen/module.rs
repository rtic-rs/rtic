use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{analyze::Analysis, check::Extra, codegen::util};

pub fn codegen(
    ctxt: Context,
    resources_tick: bool,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> TokenStream2 {
    let mut items = vec![];
    let mut fields = vec![];
    let mut values = vec![];
    // Used to copy task cfgs to the whole module
    let mut task_cfgs = vec![];

    let name = ctxt.ident(app);

    let mut needs_instant = false;
    let mut lt = None;
    match ctxt {
        Context::Init => {
            if let Some(m) = &extra.monotonic {
                fields.push(quote!(
                    /// System start time = `Instant(0 /* cycles */)`
                    pub start: <#m as rtic::Monotonic>::Instant
                ));

                values.push(quote!(start: <#m as rtic::Monotonic>::zero()));

                fields.push(quote!(
                    /// Core (Cortex-M) peripherals minus the SysTick
                    pub core: rtic::Peripherals
                ));
            } else {
                fields.push(quote!(
                    /// Core (Cortex-M) peripherals
                    pub core: rtic::export::Peripherals
                ));
            }

            if extra.peripherals {
                let device = &extra.device;

                fields.push(quote!(
                    /// Device peripherals
                    pub device: #device::Peripherals
                ));

                values.push(quote!(device: #device::Peripherals::steal()));
            }

            lt = Some(quote!('a));
            fields.push(quote!(
                /// Critical section token for init
                pub cs: rtic::export::CriticalSection<#lt>
            ));

            values.push(quote!(cs: rtic::export::CriticalSection::new()));

            values.push(quote!(core));
        }

        Context::Idle => {}

        Context::HardwareTask(..) => {
            if let Some(m) = &extra.monotonic {
                fields.push(quote!(
                    /// Time at which this handler started executing
                    pub start: <#m as rtic::Monotonic>::Instant
                ));

                values.push(quote!(start: instant));

                needs_instant = true;
            }
        }

        Context::SoftwareTask(..) => {
            if let Some(m) = &extra.monotonic {
                fields.push(quote!(
                    /// The time at which this task was scheduled to run
                    pub scheduled: <#m as rtic::Monotonic>::Instant
                ));

                values.push(quote!(scheduled: instant));

                needs_instant = true;
            }
        }
    }

    if ctxt.has_locals(app) {
        let ident = util::locals_ident(ctxt, app);
        items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as Locals;
        ));
    }

    if ctxt.has_resources(app) {
        let ident = util::resources_ident(ctxt, app);
        let lt = if resources_tick {
            lt = Some(quote!('a));
            Some(quote!('a))
        } else {
            None
        };

        items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as Resources;
        ));

        fields.push(quote!(
            /// Resources this task has access to
            pub resources: Resources<#lt>
        ));

        let priority = if ctxt.is_init() {
            None
        } else {
            Some(quote!(priority))
        };
        values.push(quote!(resources: Resources::new(#priority)));
    }

    if let Context::Init = ctxt {
        let init = &app.inits.first().unwrap();
        let late_resources = util::late_resources_ident(&init.name);

        items.push(quote!(
            #[doc(inline)]
            pub use super::#late_resources as LateResources;
        ));
    }

    let doc = match ctxt {
        Context::Idle => "Idle loop",
        Context::Init => "Initialization function",
        Context::HardwareTask(_) => "Hardware task",
        Context::SoftwareTask(_) => "Software task",
    };

    let core = if ctxt.is_init() {
        if extra.monotonic.is_some() {
            Some(quote!(core: rtic::Peripherals,))
        } else {
            Some(quote!(core: rtic::export::Peripherals,))
        }
    } else {
        None
    };

    let priority = if ctxt.is_init() {
        None
    } else {
        Some(quote!(priority: &#lt rtic::export::Priority))
    };

    let instant = if needs_instant {
        let m = extra.monotonic.clone().expect("RTIC-ICE: UNREACHABLE");

        Some(quote!(, instant: <#m as rtic::Monotonic>::Instant))
    } else {
        None
    };

    items.push(quote!(
        /// Execution context
        pub struct Context<#lt> {
            #(#fields,)*
        }

        impl<#lt> Context<#lt> {
            #[inline(always)]
            pub unsafe fn new(#core #priority #instant) -> Self {
                Context {
                    #(#values,)*
                }
            }
        }
    ));

    // not sure if this is the right way, maybe its backwards,
    // that spawn_module should put in in root

    if let Context::SoftwareTask(..) = ctxt {
        let spawnee = &app.software_tasks[name];
        let priority = spawnee.args.priority;
        let t = util::spawn_t_ident(priority);
        let cfgs = &spawnee.cfgs;
        // Store a copy of the task cfgs
        task_cfgs = cfgs.clone();
        let (args, tupled, _untupled, ty) = util::regroup_inputs(&spawnee.inputs);
        let args = &args;
        let tupled = &tupled;
        let fq = util::fq_ident(name);
        let rq = util::rq_ident(priority);
        let inputs = util::inputs_ident(name);

        let app_name = &app.name;
        let app_path = quote! {crate::#app_name};

        let device = &extra.device;
        let enum_ = util::interrupt_ident();
        let interrupt = &analysis
            .interrupts
            .get(&priority)
            .expect("RTIC-ICE: interrupt identifer not found")
            .0;

        // Spawn caller
        items.push(quote!(
        #(#cfgs)*
        pub fn spawn(#(#args,)*) -> Result<(), #ty> {
            // #let_instant // do we need it?
            use rtic::Mutex as _;
            use rtic::mutex_prelude::*;

            let input = #tupled;

            unsafe {
                if let Some(index) = rtic::export::interrupt::free(|_| #app_path::#fq.dequeue()) {
                    #app_path::#inputs
                        .get_unchecked_mut(usize::from(index))
                        .as_mut_ptr()
                        .write(input);

                    rtic::export::interrupt::free(|_| {
                        #app_path::#rq.enqueue_unchecked((#app_path::#t::#name, index));
                    });

                    rtic::pend(#device::#enum_::#interrupt);

                    Ok(())
                } else {
                    Err(input)
                }
            }

        }));

        // Schedule caller
        if let Some(m) = &extra.monotonic {
            let instants = util::instants_ident(name);

            let tq = util::tq_ident();
            let t = util::schedule_t_ident();

            items.push(quote!(
            #(#cfgs)*
            pub fn schedule(
                instant: <#m as rtic::Monotonic>::Instant
                #(,#args)*
            ) -> Result<(), #ty> {
                unsafe {
                    use rtic::Mutex as _;
                    use rtic::mutex_prelude::*;

                    let input = #tupled;
                    if let Some(index) = rtic::export::interrupt::free(|_| #app_path::#fq.dequeue()) {
                        #app_path::#inputs
                            .get_unchecked_mut(usize::from(index))
                            .as_mut_ptr()
                            .write(input);

                        #app_path::#instants
                            .get_unchecked_mut(usize::from(index))
                            .as_mut_ptr()
                            .write(instant);

                        let nr = rtic::export::NotReady {
                            instant,
                            index,
                            task: #app_path::#t::#name,
                        };

                        rtic::export::interrupt::free(|_| #app_path::#tq.enqueue_unchecked(nr));

                        Ok(())
                    } else {
                        Err(input)
                    }
                }
            }));
        }
    }

    if !items.is_empty() {
        let user_imports = &app.user_imports;

        quote!(
            #[allow(non_snake_case)]
            #(#task_cfgs)*
            #[doc = #doc]
            pub mod #name {
                #(
                    #[allow(unused_imports)]
                    #user_imports
                )*
                #(#items)*
            }
        )
    } else {
        quote!()
    }
}
