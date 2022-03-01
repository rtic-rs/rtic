use crate::{analyze::Analysis, check::Extra, codegen::util};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

#[allow(clippy::too_many_lines)]
pub fn codegen(
    ctxt: Context,
    shared_resources_tick: bool,
    local_resources_tick: bool,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> TokenStream2 {
    let mut items = vec![];
    let mut module_items = vec![];
    let mut fields = vec![];
    let mut values = vec![];
    // Used to copy task cfgs to the whole module
    let mut task_cfgs = vec![];

    let name = ctxt.ident(app);

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

        Context::Idle | Context::HardwareTask(_) | Context::SoftwareTask(_) => {}
    }

    // if ctxt.has_locals(app) {
    //     let ident = util::locals_ident(ctxt, app);
    //     module_items.push(quote!(
    //         #[doc(inline)]
    //         pub use super::#ident as Locals;
    //     ));
    // }

    if ctxt.has_local_resources(app) {
        let ident = util::local_resources_ident(ctxt, app);
        let lt = if local_resources_tick {
            lt = Some(quote!('a));
            Some(quote!('a))
        } else {
            None
        };

        module_items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as LocalResources;
        ));

        fields.push(quote!(
            /// Local Resources this task has access to
            pub local: #name::LocalResources<#lt>
        ));

        values.push(quote!(local: #name::LocalResources::new()));
    }

    if ctxt.has_shared_resources(app) {
        let ident = util::shared_resources_ident(ctxt, app);
        let lt = if shared_resources_tick {
            lt = Some(quote!('a));
            Some(quote!('a))
        } else {
            None
        };

        module_items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as SharedResources;
        ));

        fields.push(quote!(
            /// Shared Resources this task has access to
            pub shared: #name::SharedResources<#lt>
        ));

        let priority = if ctxt.is_init() {
            None
        } else {
            Some(quote!(priority))
        };
        values.push(quote!(shared: #name::SharedResources::new(#priority)));
    }

    if let Context::Init = ctxt {
        let monotonic_types: Vec<_> = app
            .monotonics
            .iter()
            .map(|(_, monotonic)| {
                let mono = &monotonic.ty;
                quote! {#mono}
            })
            .collect();

        let internal_monotonics_ident = util::mark_internal_name("Monotonics");

        items.push(quote!(
            /// Monotonics used by the system
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            pub struct #internal_monotonics_ident(
                #(pub #monotonic_types),*
            );
        ));

        module_items.push(quote!(
            pub use super::#internal_monotonics_ident as Monotonics;
        ));
    }

    let doc = match ctxt {
        Context::Idle => "Idle loop",
        Context::Init => "Initialization function",
        Context::HardwareTask(_) => "Hardware task",
        Context::SoftwareTask(_) => "Software task",
    };

    let v = Vec::new();
    let cfgs = match ctxt {
        Context::HardwareTask(t) => {
            &app.hardware_tasks[t].cfgs
            // ...
        }
        Context::SoftwareTask(t) => {
            &app.software_tasks[t].cfgs
            // ...
        }
        _ => &v,
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

    let internal_context_name = util::internal_task_ident(name, "Context");

    items.push(quote!(
        #(#cfgs)*
        /// Execution context
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        pub struct #internal_context_name<#lt> {
            #(#fields,)*
        }

        #(#cfgs)*
        impl<#lt> #internal_context_name<#lt> {
            #[inline(always)]
            pub unsafe fn new(#core #priority) -> Self {
                #internal_context_name {
                    #(#values,)*
                }
            }
        }
    ));

    module_items.push(quote!(
        #(#cfgs)*
        pub use super::#internal_context_name as Context;
    ));

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
        let rq = util::rq_ident(priority);
        let inputs = util::inputs_ident(name);

        let device = &extra.device;
        let enum_ = util::interrupt_ident();
        let interrupt = &analysis
            .interrupts
            .get(&priority)
            .expect("RTIC-ICE: interrupt identifer not found")
            .0;

        let internal_spawn_ident = util::internal_task_ident(name, "spawn");

        // Spawn caller
        items.push(quote!(

        #(#cfgs)*
        /// Spawns the task directly
        pub fn #internal_spawn_ident(#(#args,)*) -> Result<(), #ty> {
            let input = #tupled;

            unsafe {
                if let Some(index) = rtic::export::interrupt::free(|_| (&mut *#fq.get_mut()).dequeue()) {
                    (&mut *#inputs
                        .get_mut())
                        .get_unchecked_mut(usize::from(index))
                        .as_mut_ptr()
                        .write(input);

                    rtic::export::interrupt::free(|_| {
                        (&mut *#rq.get_mut()).enqueue_unchecked((#t::#name, index));
                    });

                    rtic::pend(#device::#enum_::#interrupt);

                    Ok(())
                } else {
                    Err(input)
                }
            }

        }));

        module_items.push(quote!(
            #(#cfgs)*
            pub use super::#internal_spawn_ident as spawn;
        ));

        // Schedule caller
        for (_, monotonic) in &app.monotonics {
            let instants = util::monotonic_instants_ident(name, &monotonic.ident);
            let monotonic_name = monotonic.ident.to_string();

            let tq = util::tq_ident(&monotonic.ident.to_string());
            let t = util::schedule_t_ident();
            let m = &monotonic.ident;
            let m_ident = util::monotonic_ident(&monotonic_name);
            let m_isr = &monotonic.args.binds;
            let enum_ = util::interrupt_ident();
            let spawn_handle_string = format!("{}::SpawnHandle", m);

            let (enable_interrupt, pend) = if &*m_isr.to_string() == "SysTick" {
                (
                    quote!(core::mem::transmute::<_, rtic::export::SYST>(()).enable_interrupt()),
                    quote!(rtic::export::SCB::set_pendst()),
                )
            } else {
                let rt_err = util::rt_err_ident();
                (
                    quote!(rtic::export::NVIC::unmask(#rt_err::#enum_::#m_isr)),
                    quote!(rtic::pend(#rt_err::#enum_::#m_isr)),
                )
            };

            let tq_marker = &util::timer_queue_marker_ident();

            // For future use
            // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
            // items.push(quote!(#[doc = #doc]));
            let internal_spawn_handle_ident =
                util::internal_monotonics_ident(name, m, "SpawnHandle");
            let internal_spawn_at_ident = util::internal_monotonics_ident(name, m, "spawn_at");
            let internal_spawn_after_ident =
                util::internal_monotonics_ident(name, m, "spawn_after");

            if monotonic.args.default {
                module_items.push(quote!(
                    pub use #m::spawn_after;
                    pub use #m::spawn_at;
                    pub use #m::SpawnHandle;
                ));
            }
            module_items.push(quote!(
                pub mod #m {
                    pub use super::super::#internal_spawn_after_ident as spawn_after;
                    pub use super::super::#internal_spawn_at_ident as spawn_at;
                    pub use super::super::#internal_spawn_handle_ident as SpawnHandle;
                }
            ));

            items.push(quote!(
                #(#cfgs)*
                #[allow(non_snake_case)]
                #[allow(non_camel_case_types)]
                pub struct #internal_spawn_handle_ident {
                    #[doc(hidden)]
                    marker: u32,
                }

                impl core::fmt::Debug for #internal_spawn_handle_ident {
                    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        f.debug_struct(#spawn_handle_string).finish()
                    }
                }

                #(#cfgs)*
                impl #internal_spawn_handle_ident {
                    pub fn cancel(self) -> Result<#ty, ()> {
                        rtic::export::interrupt::free(|_| unsafe {
                            let tq = &mut *#tq.get_mut();
                            if let Some((_task, index)) = tq.cancel_marker(self.marker) {
                                // Get the message
                                let msg = (&*#inputs
                                    .get())
                                    .get_unchecked(usize::from(index))
                                    .as_ptr()
                                    .read();
                                // Return the index to the free queue
                                (&mut *#fq.get_mut()).split().0.enqueue_unchecked(index);

                                Ok(msg)
                            } else {
                                Err(())
                            }
                        })
                    }

                    #[inline]
                    pub fn reschedule_after(
                        self,
                        duration: <#m as rtic::Monotonic>::Duration
                    ) -> Result<Self, ()> {
                        self.reschedule_at(monotonics::#m::now() + duration)
                    }

                    pub fn reschedule_at(
                        self,
                        instant: <#m as rtic::Monotonic>::Instant
                    ) -> Result<Self, ()> {
                        rtic::export::interrupt::free(|_| unsafe {
                            let marker = #tq_marker.get().read();
                            #tq_marker.get_mut().write(marker.wrapping_add(1));

                            let tq = (&mut *#tq.get_mut());

                            tq.update_marker(self.marker, marker, instant, || #pend).map(|_| #name::#m::SpawnHandle { marker })
                        })
                    }
                }

                #(#cfgs)*
                /// Spawns the task after a set duration relative to the current time
                ///
                /// This will use the time `Instant::new(0)` as baseline if called in `#[init]`,
                /// so if you use a non-resetable timer use `spawn_at` when in `#[init]`
                #[allow(non_snake_case)]
                pub fn #internal_spawn_after_ident(
                    duration: <#m as rtic::Monotonic>::Duration
                    #(,#args)*
                ) -> Result<#name::#m::SpawnHandle, #ty>
                {
                    let instant = monotonics::#m::now();

                    #internal_spawn_at_ident(instant + duration #(,#untupled)*)
                }

                #(#cfgs)*
                /// Spawns the task at a fixed time instant
                #[allow(non_snake_case)]
                pub fn #internal_spawn_at_ident(
                    instant: <#m as rtic::Monotonic>::Instant
                    #(,#args)*
                ) -> Result<#name::#m::SpawnHandle, #ty> {
                    unsafe {
                        let input = #tupled;
                        if let Some(index) = rtic::export::interrupt::free(|_| (&mut *#fq.get_mut()).dequeue()) {
                            (&mut *#inputs
                                .get_mut())
                                .get_unchecked_mut(usize::from(index))
                                .as_mut_ptr()
                                .write(input);

                            (&mut *#instants
                                .get_mut())
                                .get_unchecked_mut(usize::from(index))
                                .as_mut_ptr()
                                .write(instant);

                            rtic::export::interrupt::free(|_| {
                                let marker = #tq_marker.get().read();
                                let nr = rtic::export::NotReady {
                                    instant,
                                    index,
                                    task: #t::#name,
                                    marker,
                                };

                                #tq_marker.get_mut().write(#tq_marker.get().read().wrapping_add(1));

                                let tq = &mut *#tq.get_mut();

                                tq.enqueue_unchecked(
                                    nr,
                                    || #enable_interrupt,
                                    || #pend,
                                    (&mut *#m_ident.get_mut()).as_mut());

                                Ok(#name::#m::SpawnHandle { marker })
                            })
                        } else {
                            Err(input)
                        }
                    }
                }
            ));
        }
    }

    if items.is_empty() {
        quote!()
    } else {
        quote!(
            #(#items)*

            #[allow(non_snake_case)]
            #(#task_cfgs)*
            #[doc = #doc]
            pub mod #name {
                #(#module_items)*
            }
        )
    }
}
