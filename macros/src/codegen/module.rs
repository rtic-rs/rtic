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
    let app_name = &app.name;
    let app_path = quote! {crate::#app_name};

    let all_task_imports: Vec<_> = app
        .software_tasks
        .iter()
        .map(|(name, st)| {
            if !st.is_extern {
                let cfgs = &st.cfgs;
                quote! {
                    #(#cfgs)*
                    #[allow(unused_imports)]
                    use #app_path::#name as #name;
                }
            } else {
                quote!()
            }
        })
        .chain(app.hardware_tasks.iter().map(|(name, ht)| {
            if !ht.is_extern {
                quote! {
                    #[allow(unused_imports)]
                    use #app_path::#name as #name;
                }
            } else {
                quote!()
            }
        }))
        .chain(app.user_types.iter().map(|ty| {
            let t = &ty.ident;
            quote! {
                #[allow(unused_imports)]
                use super::#t;
            }
        }))
        .collect();

    let mut lt = None;
    match ctxt {
        Context::Init => {
            fields.push(quote!(
                /// Core (Cortex-M) peripherals
                pub core: rtic::export::Peripherals
            ));

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

        Context::HardwareTask(_) => {}

        Context::SoftwareTask(_) => {}
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
        let ident = util::mark_internal_ident(&ident);
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
        let late_fields = analysis
            .late_resources
            .iter()
            .flat_map(|resources| {
                resources.iter().map(|name| {
                    let ty = &app.late_resources[name].ty;
                    let cfgs = &app.late_resources[name].cfgs;

                    quote!(
                        #(#cfgs)*
                        pub #name: #ty
                    )
                })
            })
            .collect::<Vec<_>>();

        items.push(quote!(
            /// Resources initialized at runtime
            #[allow(non_snake_case)]
            pub struct LateResources {
                #(#late_fields),*
            }
        ));

        let monotonic_types: Vec<_> = app
            .monotonics
            .iter()
            .map(|(_, monotonic)| {
                let mono = &monotonic.ty;
                quote! {#mono}
            })
            .collect();

        items.push(quote!(
            /// Monotonics used by the system
            #[allow(non_snake_case)]
            pub struct Monotonics(
                #(pub #monotonic_types),*
            );
        ));
    }

    let doc = match ctxt {
        Context::Idle => "Idle loop",
        Context::Init => "Initialization function",
        Context::HardwareTask(_) => "Hardware task",
        Context::SoftwareTask(_) => "Software task",
    };

    let core = if ctxt.is_init() {
        Some(quote!(core: rtic::export::Peripherals,))
    } else {
        None
    };

    let priority = if ctxt.is_init() {
        None
    } else {
        Some(quote!(priority: &#lt rtic::export::Priority))
    };

    items.push(quote!(
        /// Execution context
        pub struct Context<#lt> {
            #(#fields,)*
        }

        impl<#lt> Context<#lt> {
            #[inline(always)]
            pub unsafe fn new(#core #priority) -> Self {
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
        let (args, tupled, untupled, ty) = util::regroup_inputs(&spawnee.inputs);
        let args = &args;
        let tupled = &tupled;
        let fq = util::fq_ident(name);
        let fq = util::mark_internal_ident(&fq);
        let rq = util::rq_ident(priority);
        let rq = util::mark_internal_ident(&rq);
        let inputs = util::inputs_ident(name);
        let inputs = util::mark_internal_ident(&inputs);

        let device = &extra.device;
        let enum_ = util::interrupt_ident();
        let interrupt = &analysis
            .interrupts
            .get(&priority)
            .expect("RTIC-ICE: interrupt identifer not found")
            .0;

        // Spawn caller
        items.push(quote!(

        #(#all_task_imports)*

        #(#cfgs)*
        /// Spawns the task directly
        pub fn spawn(#(#args,)*) -> Result<(), #ty> {
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
        for (_, monotonic) in &app.monotonics {
            let instants = util::monotonic_instants_ident(name, &monotonic.ident);
            let instants = util::mark_internal_ident(&instants);
            let monotonic_name = monotonic.ident.to_string();

            let tq = util::tq_ident(&monotonic.ident.to_string());
            let tq = util::mark_internal_ident(&tq);
            let t = util::schedule_t_ident();
            let m = &monotonic.ident;
            let mono_type = &monotonic.ident;
            let m_ident = util::monotonic_ident(&monotonic_name);
            let m_ident = util::mark_internal_ident(&m_ident);
            let m_isr = &monotonic.args.binds;
            let enum_ = util::interrupt_ident();

            if monotonic.args.default {
                items.push(quote!(pub use #m::spawn_after;));
                items.push(quote!(pub use #m::spawn_at;));
                items.push(quote!(pub use #m::SpawnHandle;));
            }

            let (enable_interrupt, pend) = if &*m_isr.to_string() == "SysTick" {
                (
                    quote!(core::mem::transmute::<_, cortex_m::peripheral::SYST>(())
                        .enable_interrupt()),
                    quote!(cortex_m::peripheral::SCB::set_pendst()),
                )
            } else {
                let rt_err = util::rt_err_ident();
                (
                    quote!(rtic::export::NVIC::unmask(#app_path::#rt_err::#enum_::#m_isr)),
                    quote!(rtic::pend(#app_path::#rt_err::#enum_::#m_isr)),
                )
            };

            let user_imports = &app.user_imports;
            let tq_marker = util::mark_internal_ident(&util::timer_queue_marker_ident());

            items.push(quote!(
            /// Holds methods related to this monotonic
            pub mod #m {
                use super::*;
                #[allow(unused_imports)]
                use #app_path::#tq_marker;
                #[allow(unused_imports)]
                use #app_path::#t;
                #(
                    #[allow(unused_imports)]
                    #user_imports
                )*

                pub struct SpawnHandle {
                    #[doc(hidden)]
                    marker: u32,
                }

                impl SpawnHandle {
                    pub fn cancel(self) -> Result<#ty, ()> {
                        rtic::export::interrupt::free(|_| unsafe {
                            let tq = &mut *#app_path::#tq.as_mut_ptr();
                            if let Some((_task, index)) = tq.cancel_marker(self.marker) {
                                // Get the message
                                let msg = #app_path::#inputs.get_unchecked(usize::from(index)).as_ptr().read();
                                // Return the index to the free queue
                                #app_path::#fq.split().0.enqueue_unchecked(index);

                                Ok(msg)
                            } else {
                                Err(())
                            }
                        })
                    }

                    #[inline]
                    pub fn reschedule_after<D>(self, duration: D) -> Result<Self, ()>
                        where D: rtic::time::duration::Duration + rtic::time::fixed_point::FixedPoint,
                                 D::T: Into<<#app_path::#mono_type as rtic::time::Clock>::T>,
                    {
                        self.reschedule_at(#app_path::monotonics::#m::now() + duration)
                    }

                    pub fn reschedule_at(self, instant: rtic::time::Instant<#app_path::#mono_type>) -> Result<Self, ()>
                    {
                        rtic::export::interrupt::free(|_| unsafe {
                            let marker = #tq_marker;
                            #tq_marker = #tq_marker.wrapping_add(1);

                            let tq = &mut *#app_path::#tq.as_mut_ptr();

                            tq.update_marker(self.marker, marker, instant, || #pend).map(|_| SpawnHandle { marker })
                        })
                    }
                }

                #(#cfgs)*
                /// Spawns the task after a set duration relative to the current time
                ///
                /// This will use the time `Instant::new(0)` as baseline if called in `#[init]`,
                /// so if you use a non-resetable timer use `spawn_at` when in `#[init]`
                pub fn spawn_after<D>(
                    duration: D
                    #(,#args)*
                ) -> Result<SpawnHandle, #ty>
                    where D: rtic::time::duration::Duration + rtic::time::fixed_point::FixedPoint,
                        D::T: Into<<#app_path::#mono_type as rtic::time::Clock>::T>,
                {

                    let instant = if rtic::export::interrupt::free(|_| unsafe { #app_path::#m_ident.is_none() }) {
                        rtic::time::Instant::new(0)
                    } else {
                        #app_path::monotonics::#m::now()
                    };

                    spawn_at(instant + duration #(,#untupled)*)
                }

                #(#cfgs)*
                /// Spawns the task at a fixed time instant
                pub fn spawn_at(
                    instant: rtic::time::Instant<#app_path::#mono_type>
                    #(,#args)*
                ) -> Result<SpawnHandle, #ty> {
                    unsafe {
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

                            rtic::export::interrupt::free(|_| {
                                let marker = #tq_marker;
                                let nr = rtic::export::NotReady {
                                    instant,
                                    index,
                                    task: #app_path::#t::#name,
                                    marker,
                                };

                                #tq_marker = #tq_marker.wrapping_add(1);

                                let tq = unsafe { &mut *#app_path::#tq.as_mut_ptr() };

                                if let Some(mono) = #app_path::#m_ident.as_mut() {
                                    tq.enqueue_unchecked(
                                        nr,
                                        || #enable_interrupt,
                                        || #pend,
                                        mono)
                                } else {
                                    // We can only use the timer queue if `init` has returned, and it
                                    // writes the `Some(monotonic)` we are accessing here.
                                    core::hint::unreachable_unchecked()
                                }

                                Ok(SpawnHandle { marker })
                            })
                        } else {
                            Err(input)
                        }
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
