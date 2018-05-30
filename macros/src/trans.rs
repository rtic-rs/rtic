use either::Either;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::Ident;
use syntax::check::App;

use analyze::{Context, Ceiling};

pub fn app(ctxt: &Context, app: &App) -> TokenStream {
    let mut root = vec![];
    let k = Ident::new("_rtfm", Span::call_site());
    let device = &app.device;

    let needs_tq = !ctxt.schedule_after.is_empty();

    root.push(quote! {
        extern crate cortex_m_rtfm as #k;
        #[allow(unused_imports)]
        use #k::Resource as _cortex_m_rtfm_Resource;
    });

    /* Resources */
    let mut resources = vec![];
    for (name, resource) in &app.resources {
        if app.init.resources.contains(name) {
            // `init` resources are handled below
            continue;
        }

        let ty = &resource.ty;
        let expr = resource
            .expr
            .as_ref()
            .map(|e| quote!(#e))
            .unwrap_or_else(|| quote!(unsafe { ::#k::_impl::uninitialized() }));

        let (shared, priority) = ctxt.ceilings
            .resources()
            .get(name)
            .cloned()
            .map(|ceiling| match ceiling {
                Ceiling::Owned(p) => (false, p),
                Ceiling::Shared(p) => (true, p),
            })
            .unwrap_or( (false, 0) ); // 0 = resource owned by `init`

        let ceiling = Ident::new(&format!("U{}", priority), Span::call_site());

        root.push(quote! {
            #[allow(unsafe_code)]
            unsafe impl ::#k::Resource for _resource::#name {
                const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                type Ceiling = ::#k::_impl::#ceiling;
                type Data = #ty;

                unsafe fn _var() -> &'static mut Self::Data {
                    static mut #name: #ty = #expr;

                    &mut #name
                }
            }
        });

        if shared {
            root.push(quote! {
                impl ::#k::SharedResource for _resource::#name {}
            })
        }

        resources.push(quote! {
            #[allow(non_camel_case_types)]
            pub struct #name { _not_send_or_sync: PhantomData<*const ()> }

            #[allow(dead_code)]
            #[allow(unsafe_code)]
            impl #name {
                pub unsafe fn new() -> Self {
                    #name { _not_send_or_sync: PhantomData }
                }
            }
        });
    }

    root.push(quote! {
        mod _resource {
            #[allow(unused_imports)]
            use core::marker::PhantomData;

            #(#resources)*
        }
    });

    /* "resources" owned by `init` */
    // These don't implement the `Resource` trait because they are never shared. Instead they
    // implement the `Singleton` trait and may or may not appear wrapped in `Uninit`
    for (name, resource) in &app.resources {
        if !app.init.resources.contains(name) {
            // `init` resources are handled below
            continue;
        }

        let ty = &resource.ty;
        let expr = resource
            .expr
            .as_ref()
            .map(|e| quote!(#e))
            .unwrap_or_else(|| quote!(unsafe { #k::_impl::uninitialized() }));

        // TODO replace this with a call to `heapless::singleton!` when it doesn't require a feature
        // gate in the user code
        root.push(quote! {
            pub struct #name { _private: #k::_impl::Private }

            #[allow(unsafe_code)]
            unsafe impl #k::_impl::Singleton for #name {
                type Data = #ty;

                unsafe fn _var() -> &'static mut #ty {
                    static mut VAR: #ty = #expr;

                    &mut VAR
                }
            }

            #[allow(unsafe_code)]
            impl AsRef<#ty> for #name {
                fn as_ref(&self) -> &#ty {
                    use #k::_impl::Singleton;

                    unsafe { #name::_var() }
                }
            }

            #[allow(unsafe_code)]
            impl AsMut<#ty> for #name {
                fn as_mut(&mut self) -> &mut #ty {
                    use #k::_impl::Singleton;

                    unsafe { #name::_var() }
                }
            }

            impl core::ops::Deref for #name {
                type Target = #ty;

                fn deref(&self) -> &#ty {
                    self.as_ref()
                }
            }

            impl core::ops::DerefMut for #name {
                fn deref_mut(&mut self) -> &mut #ty {
                    self.as_mut()
                }
            }

            #[allow(unsafe_code)]
            impl Into<&'static mut #ty> for #name {
                fn into(self) -> &'static mut #ty {
                    use #k::_impl::Singleton;

                    unsafe { #name::_var() }
                }
            }

            #[allow(unsafe_code)]
            unsafe impl #k::_impl::StableDeref for #name {}
        });

        if resource.expr.is_some() {
            root.push(quote! {
                impl #name {
                    #[allow(unsafe_code)]
                    unsafe fn _new() -> Self {
                        #name { _private: #k::_impl::Private::new() }
                    }
                }
            });
        } else {
            root.push(quote! {
                impl #name {
                    #[allow(unsafe_code)]
                    unsafe fn _new() -> #k::_impl::Uninit<Self> {
                        #k::_impl::Uninit::new(#name { _private: #k::_impl::Private::new() })
                    }
                }
            });
        }
    }

    /* Tasks */
    for (name, task) in &app.tasks {
        let path = &task.path;

        let lifetime = if task
            .resources
            .iter()
            .any(|res| ctxt.ceilings.resources()[res].is_owned())
        {
            Some(quote!('a))
        } else {
            None
        };

        let _context = Ident::new(
            &format!("_ZN{}{}7ContextE", name.to_string().as_bytes().len(), name),
            Span::call_site(),
        );

        let mut mod_ = vec![];

        let time_field = if task.interrupt_or_instances.is_left() {
            quote!(start_time)
        } else {
            quote!(scheduled_time)
        };

        let input_ = task
            .input
            .as_ref()
            .map(|input| quote!(#input))
            .unwrap_or(quote!(()));

        // NOTE some stuff has to go in the root because `#input` is not guaranteed to be a
        // primitive type and there's no way to import that type into a module (we don't know its
        // full path). So instead we just assume that `#input` has been imported in the root; this
        // forces us to put anything that refers to `#input` in the root.
        if cfg!(feature = "timer-queue") {
            root.push(quote! {
                pub struct #_context<#lifetime> {
                    pub #time_field: u32,
                    pub input: #input_,
                    pub resources: #name::Resources<#lifetime>,
                    pub tasks: #name::Tasks,
                    pub priority: ::#k::Priority<#name::Priority>,
                }

                #[allow(unsafe_code)]
                impl<#lifetime> #_context<#lifetime> {
                    pub unsafe fn new(bl: ::#k::_impl::Instant, payload: #input_) -> Self {
                        #_context {
                            tasks: #name::Tasks::new(bl),
                            #time_field: bl.into(),
                            input: payload,
                            resources: #name::Resources::new(),
                            priority: ::#k::Priority::_new(),
                        }
                    }
                }
            });
        } else {
            root.push(quote! {
                pub struct #_context<#lifetime> {
                    pub tasks: #name::Tasks,
                    pub input: #input_,
                    pub resources: #name::Resources<#lifetime>,
                    pub priority: ::#k::Priority<#name::Priority>,
                }

                #[allow(unsafe_code)]
                impl<#lifetime> #_context<#lifetime> {
                    pub unsafe fn new(payload: #input_) -> Self {
                        #_context {
                            tasks: #name::Tasks::new(),
                            input: payload,
                            resources: #name::Resources::new(),
                            priority: ::#k::Priority::_new(),
                        }
                    }
                }
            });
        }

        let res_fields = task
            .resources
            .iter()
            .map(|res| {
                if ctxt.ceilings.resources()[res].is_owned() {
                    let ty = &app.resources[res].ty;
                    quote!(pub #res: &'a mut #ty)
                } else {
                    quote!(pub #res: ::_resource::#res)
                }
            })
            .collect::<Vec<_>>();

        let res_exprs = task.resources.iter().map(|res| {
            if ctxt.ceilings.resources()[res].is_owned() {
                quote!(#res: ::_resource::#res::_var())
            } else {
                quote!(#res: ::_resource::#res::new())
            }
        });

        let tasks_fields = task
            .schedule_now
            .iter()
            .map(|task| quote!(pub #task: ::_schedule_now::#task))
            .chain(
                task.schedule_after
                    .iter()
                    .map(|task| quote!(pub #task: ::_schedule_after::#task)),
            )
            .collect::<Vec<_>>();

        let tasks_exprs = task
            .schedule_now
            .iter()
            .map(|task| {
                if cfg!(feature = "timer-queue") {
                    quote!(#task: ::_schedule_now::#task::new(_bl))
                } else {
                    quote!(#task: ::_schedule_now::#task::new())
                }
            })
            .chain(
                task.schedule_after
                    .iter()
                    .map(|task| quote!(#task: ::_schedule_after::#task::new(_bl))),
            )
            .collect::<Vec<_>>();

        let priority = Ident::new(&format!("U{}", task.priority), Span::call_site());
        mod_.push(quote! {
            #[allow(unused_imports)]
            use ::#k::Resource;

            pub const HANDLER: fn(Context) = ::#path;

            // The priority at this task is dispatched at
            pub type Priority = ::#k::_impl::#priority;

            pub use ::#_context as Context;

            pub struct Tasks {
                #(#tasks_fields,)*
            }

            #[allow(non_snake_case)]
            pub struct Resources<#lifetime> {
                #(#res_fields,)*
            }

            #[allow(unsafe_code)]
            impl<#lifetime> Resources<#lifetime> {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#res_exprs,)*
                    }
                }
            }
        });

        if cfg!(feature = "timer-queue") {
            mod_.push(quote! {
                #[allow(unsafe_code)]
                impl Tasks {
                    pub unsafe fn new(_bl: ::#k::_impl::Instant) -> Self {
                        Tasks {
                            #(#tasks_exprs,)*
                        }
                    }
                }
            });
        } else {
            mod_.push(quote! {
                #[allow(unsafe_code)]
                impl Tasks {
                    pub unsafe fn new() -> Self {
                        Tasks {
                            #(#tasks_exprs,)*
                        }
                    }
                }
            });
        }

        match task.interrupt_or_instances {
            Either::Left(ref interrupt) => {
                let export_name = interrupt.to_string();
                let fn_name = Ident::new(&format!("_{}", interrupt), Span::call_site());

                let bl = if cfg!(feature = "timer-queue") {
                    Some(quote!(_now,))
                } else {
                    None
                };

                root.push(quote! {
                    #[allow(non_snake_case)]
                    #[allow(unsafe_code)]
                    #[export_name = #export_name]
                    pub unsafe extern "C" fn #fn_name() {
                        use #device::Interrupt;
                        let _ = Interrupt::#interrupt; // verify that the interrupt exists
                        let _now = ::#k::_impl::Instant::now();
                        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
                        #name::HANDLER(#name::Context::new(#bl ()))
                    }
                });
            }
            Either::Right(instances) => {
                let ucapacity = Ident::new(&format!("U{}", instances), Span::call_site());
                let capacity = instances as usize;

                root.push(quote! {
                    #[allow(unsafe_code)]
                    unsafe impl ::#k::Resource for #name::FREE_QUEUE {
                        const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                        type Ceiling = #name::Ceiling;
                        type Data = ::#k::_impl::FreeQueue<::#k::_impl::#ucapacity>;

                        unsafe fn _var() -> &'static mut Self::Data {
                            static mut FREE_QUEUE:
                                ::#k::_impl::FreeQueue<::#k::_impl::#ucapacity> =
                                ::#k::_impl::FreeQueue::u8();

                            &mut FREE_QUEUE
                        }
                    }

                    impl ::#k::SharedResource for #name::FREE_QUEUE {}

                });

                let ceiling = Ident::new(
                    &format!(
                        "U{}",
                        ctxt.ceilings.slot_queues().get(name).cloned() // 0 = owned by init
                        .unwrap_or(0)
                    ),
                    Span::call_site(),
                );

                let mangled = Ident::new(
                    &format!("_ZN{}{}6PAYLOADSE", name.to_string().len(), name),
                    Span::call_site(),
                );

                // NOTE must be in the root because of `#input`
                root.push(quote! {
                    #[allow(non_upper_case_globals)]
                    #[allow(unsafe_code)]
                    pub static mut #mangled: [#input_; #capacity] =
                        unsafe { ::#k::_impl::uninitialized() };

                });

                mod_.push(quote! {
                    pub use ::#mangled as PAYLOADS;

                    #[allow(dead_code)]
                    #[allow(unsafe_code)]
                    pub static mut SCHEDULED_TIMES: [::#k::_impl::Instant; #capacity] = unsafe {
                        ::#k::_impl::uninitialized()
                    };

                    #[allow(non_camel_case_types)]
                    pub struct FREE_QUEUE { _0: () }

                    #[allow(dead_code)]
                    #[allow(unsafe_code)]
                    impl FREE_QUEUE {
                        pub unsafe fn new() -> Self {
                            FREE_QUEUE { _0: () }
                        }
                    }

                    // Ceiling of the `FREE_QUEUE` resource
                    pub type Ceiling = ::#k::_impl::#ceiling;
                });
            }
        }

        root.push(quote! {
            mod #name {
                #(#mod_)*
            }
        });
    }

    /* schedule_now */
    let schedule_now = ctxt
        .schedule_now
        .iter()
        .map(|name| {
            let task = &app.tasks[name];
            let priority = task.priority;
            let _priority = Ident::new(&format!("_{}", priority), Span::call_site());
            let interrupt = ctxt.dispatchers[&priority].interrupt();

            let input_ = task
                .input
                .as_ref()
                .map(|input| quote!(#input))
                .unwrap_or(quote!(()));
            let (payload_in, payload_out) = if let Some(input) = task.input.as_ref() {
                (quote!(payload: #input,), quote!(payload))
            } else {
                (quote!(), quote!(()))
            };

            let sqc = Ident::new(
                &format!(
                    "U{}",
                    ctxt.ceilings.slot_queues().get(name).cloned() // 0 = owned by init
                    .unwrap_or(0)
                ),
                Span::call_site(),
            );
            let qc = Ident::new(
                &format!("U{}", ctxt.ceilings.dispatch_queues()[&priority]),
                Span::call_site(),
            );

            if cfg!(feature = "timer-queue") {
                root.push(quote! {
                    #[allow(dead_code)]
                    #[allow(unsafe_code)]
                    impl _schedule_now::#name {
                        #[inline]
                        pub fn schedule_now<P>(
                            &mut self,
                            _p: &::#k::Priority<P>,
                            #payload_in
                        ) -> Result<(), #input_>
                        where
                            P: ::#k::_impl::Unsigned +
                                ::#k::_impl::Max<::#k::_impl::#sqc> +
                                ::#k::_impl::Max<::#k::_impl::#qc>,
                            ::#k::_impl::Maximum<P, ::#k::_impl::#sqc>: ::#k::_impl::Unsigned,
                            ::#k::_impl::Maximum<P, ::#k::_impl::#qc>: ::#k::_impl::Unsigned,
                        {
                            unsafe {
                                use ::#k::Resource;

                                let p: &mut ::#k::Priority<P> = &mut ::#k::Priority::_new();
                                let slot = ::#name::FREE_QUEUE::new()
                                    .claim_mut(p, |sq, _| sq.dequeue());
                                if let Some(index) = slot {
                                    let task = ::#_priority::Task::#name;
                                    core::ptr::write(
                                        #name::PAYLOADS.get_unchecked_mut(index as usize),
                                        #payload_out,
                                    );
                                    *#name::SCHEDULED_TIMES.get_unchecked_mut(index as usize) =
                                        self.scheduled_time();

                                    #_priority::READY_QUEUE::new().claim_mut(p, |q, _| {
                                        q.split().0.enqueue_unchecked((task, index));
                                    });

                                    use #device::Interrupt;
                                    ::#k::_impl::trigger(Interrupt::#interrupt);

                                    Ok(())
                                } else {
                                    Err(#payload_out)
                                }
                            }
                        }
                    }
                });

                quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #name { scheduled_time: ::#k::_impl::Instant }

                    #[allow(dead_code)]
                    #[allow(unsafe_code)]
                    impl #name {
                        pub unsafe fn new(bl: ::#k::_impl::Instant) -> Self {
                            #name { scheduled_time: bl }
                        }

                        pub fn scheduled_time(&self) -> ::#k::_impl::Instant {
                            self.scheduled_time
                        }
                    }
                }
            } else {
                root.push(quote! {
                    #[allow(dead_code)]
                    #[allow(unsafe_code)]
                    impl _schedule_now::#name {
                        #[inline]
                        pub fn schedule_now<P>(
                            &mut self,
                            _p: &::#k::Priority<P>,
                            #payload_in
                        ) -> Result<(), #input_>
                        where
                            P: ::#k::_impl::Unsigned +
                                ::#k::_impl::Max<::#k::_impl::#sqc> +
                                ::#k::_impl::Max<::#k::_impl::#qc>,
                            ::#k::_impl::Maximum<P, ::#k::_impl::#sqc>: ::#k::_impl::Unsigned,
                            ::#k::_impl::Maximum<P, ::#k::_impl::#qc>: ::#k::_impl::Unsigned,
                        {
                            unsafe {
                                use ::#k::Resource;

                                let p: &mut ::#k::Priority<P> = &mut ::#k::Priority::_new();
                                if let Some(index) =
                                    ::#name::FREE_QUEUE::new().claim_mut(p, |sq, _| sq.dequeue()) {
                                    let task = ::#_priority::Task::#name;
                                    core::ptr::write(
                                        ::#name::PAYLOADS.get_unchecked_mut(index as usize),
                                        #payload_out,
                                    );

                                    ::#_priority::READY_QUEUE::new().claim_mut(p, |q, _| {
                                        q.split().0.enqueue_unchecked((task, index));
                                    });

                                    use #device::Interrupt;
                                    ::#k::_impl::trigger(Interrupt::#interrupt);

                                    Ok(())
                                } else {
                                    Err(#payload_out)
                                }
                            }
                        }
                    }
                });

                quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #name {}

                    #[allow(dead_code)]
                    #[allow(unsafe_code)]
                    impl #name {
                        pub unsafe fn new() -> Self {
                            #name {}
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    root.push(quote! {
        mod _schedule_now {
            #[allow(unused_imports)]
            use ::#k::Resource;

            #(#schedule_now)*
        }
    });

    /* schedule_after */
    let schedule_after = ctxt
        .schedule_after
        .iter()
        .map(|name| {
            let task = &app.tasks[name];

            let sqc = Ident::new(
                &format!(
                    "U{}",
                    ctxt.ceilings.slot_queues().get(name).unwrap_or(&0) // 0 = owned by init
                ),
                Span::call_site(),
            );
            let tqc = Ident::new(
                &format!("U{}", ctxt.ceilings.timer_queue()),
                Span::call_site(),
            );

            let input_ = task
                .input
                .as_ref()
                .map(|input| quote!(#input))
                .unwrap_or(quote!(()));
            let (payload_in, payload_out) = if let Some(input) = task.input.as_ref() {
                (quote!(payload: #input,), quote!(payload))
            } else {
                (quote!(), quote!(()))
            };

            // NOTE needs to be in the root because of `#ty`
            root.push(quote! {
                #[allow(dead_code)]
                #[allow(unsafe_code)]
                impl _schedule_after::#name {
                    #[inline]
                    pub fn schedule_after<P>(
                        &self,
                        _p: &::#k::Priority<P>,
                        after: u32,
                        #payload_in
                    ) -> Result<(), #input_>
                    where
                        P: ::#k::_impl::Unsigned +
                            ::#k::_impl::Max<::#k::_impl::#sqc> +
                            ::#k::_impl::Max<::#k::_impl::#tqc>,
                        ::#k::_impl::Maximum<P, ::#k::_impl::#sqc>: ::#k::_impl::Unsigned,
                        ::#k::_impl::Maximum<P, ::#k::_impl::#tqc>: ::#k::_impl::Unsigned,
                    {
                        unsafe {
                            use ::#k::Resource;

                            let p: &mut ::#k::Priority<P> = &mut ::#k::Priority::_new();
                            if let Some(index) =
                                ::#name::FREE_QUEUE::new().claim_mut(p, |sq, _| sq.dequeue()) {
                                let ss = self.scheduled_time() + after;
                                let task = ::_tq::Task::#name;

                                core::ptr::write(
                                    ::#name::PAYLOADS.get_unchecked_mut(index as usize),
                                    #payload_out,
                                );

                                *::#name::SCHEDULED_TIMES.get_unchecked_mut(index as usize) = ss;

                                let m = ::#k::_impl::NotReady {
                                    scheduled_time: ss,
                                    index,
                                    task,
                                };

                                ::_tq::TIMER_QUEUE::new().claim_mut(p, |tq, _| tq.enqueue(m));

                                Ok(())
                            } else {
                                Err(#payload_out)
                            }
                        }
                    }
                }
            });

            quote! {
                #[allow(non_camel_case_types)]
                pub struct #name { scheduled_time: ::#k::_impl::Instant }

                #[allow(dead_code)]
                #[allow(unsafe_code)]
                impl #name {
                    pub unsafe fn new(ss: ::#k::_impl::Instant) -> Self {
                        #name { scheduled_time: ss }
                    }

                    pub fn scheduled_time(&self) -> ::#k::_impl::Instant {
                        self.scheduled_time
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    root.push(quote! {
        mod _schedule_after {
            #[allow(unused_imports)]
            use ::#k::Resource;

            #(#schedule_after)*
        }
    });

    /* Timer queue */
    if needs_tq {
        let capacity = Ident::new(
            &format!("U{}", ctxt.timer_queue.capacity()),
            Span::call_site(),
        );
        let tasks = ctxt.timer_queue.tasks().keys();
        let arms = ctxt
            .timer_queue
            .tasks()
            .iter()
            .map(|(name, priority)| {
                let _priority = Ident::new(&format!("_{}", priority), Span::call_site());
                let interrupt = ctxt.dispatchers[priority].interrupt();

                quote! {
                    _tq::Task::#name => {
                        #_priority::READY_QUEUE::new().claim_mut(t, |q, _| {
                            q.split().0.enqueue_unchecked((#_priority::Task::#name, index))
                        });
                        use #device::Interrupt;
                        ::#k::_impl::trigger(Interrupt::#interrupt);
                    }
                }
            })
            .collect::<Vec<_>>();

        let ceiling = Ident::new(
            &format!("U{}", ctxt.ceilings.timer_queue()),
            Span::call_site(),
        );
        let priority = Ident::new(&format!("U{}", ctxt.sys_tick), Span::call_site());
        root.push(quote! {
            mod _tq {
                #[allow(non_camel_case_types)]
                pub struct TIMER_QUEUE { _0: () }

                #[allow(unsafe_code)]
                impl TIMER_QUEUE {
                    pub unsafe fn new() -> Self {
                        TIMER_QUEUE { _0: () }
                    }
                }

                #[allow(unsafe_code)]
                unsafe impl ::#k::Resource for TIMER_QUEUE {
                    const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                    type Ceiling = ::#k::_impl::#ceiling;
                    type Data = ::#k::_impl::TimerQueue<Task, ::#k::_impl::#capacity>;

                    unsafe fn _var() -> &'static mut Self::Data {
                        static mut TIMER_QUEUE: ::#k::_impl::TimerQueue<Task, ::#k::_impl::#capacity> =
                            unsafe { ::#k::_impl::uninitialized() };

                        &mut TIMER_QUEUE
                    }
                }

                impl ::#k::SharedResource for TIMER_QUEUE;

                // SysTick priority
                pub type Priority = ::#k::_impl::#priority;

                #[allow(non_camel_case_types)]
                #[allow(dead_code)]
                #[derive(Clone, Copy)]
                pub enum Task { #(#tasks,)* }
            }

            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            #[export_name = "SysTick"]
            pub unsafe extern "C" fn _impl_SysTick() {
                use ::#k::Resource;

                ::#k::_impl::dispatch(
                    &mut ::#k::Priority::<_tq::Priority>::_new(),
                    &mut _tq::TIMER_QUEUE::new(),
                    |t, task, index| {
                        match task {
                            #(#arms,)*
                        }
                    })
            }
        });
    }

    /* Dispatchers */
    for (priority, dispatcher) in &ctxt.dispatchers {
        let _priority = Ident::new(&format!("_{}", priority), Span::call_site());
        let capacity = Ident::new(&format!("U{}", dispatcher.capacity()), Span::call_site());
        let tasks = dispatcher.tasks();
        let ceiling = Ident::new(
            &format!("U{}", ctxt.ceilings.dispatch_queues()[priority]),
            Span::call_site(),
        );

        root.push(quote! {
            mod #_priority {
                #[allow(non_camel_case_types)]
                pub struct READY_QUEUE { _0: () }

                #[allow(unsafe_code)]
                #[allow(dead_code)]
                impl READY_QUEUE {
                    pub unsafe fn new() -> Self {
                        READY_QUEUE { _0: () }
                    }
                }

                #[allow(unsafe_code)]
                unsafe impl ::#k::Resource for READY_QUEUE {
                    const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                    type Ceiling = ::#k::_impl::#ceiling;
                    type Data = ::#k::_impl::ReadyQueue<Task, ::#k::_impl::#capacity>;

                    unsafe fn _var() -> &'static mut Self::Data {
                        static mut READY_QUEUE:
                            ::#k::_impl::ReadyQueue<Task, ::#k::_impl::#capacity> =
                            ::#k::_impl::ReadyQueue::u8();

                        &mut READY_QUEUE
                    }
                }

                impl ::#k::SharedResource for READY_QUEUE {}

                #[allow(non_camel_case_types)]
                #[allow(dead_code)]
                #[derive(Clone, Copy)]
                pub enum Task { #(#tasks,)* }
            }
        });

        let arms = dispatcher
            .tasks()
            .iter()
            .map(|name| {
                // NOTE(_var) this is the only free slot producer because a task can only be
                // dispatched at one priority
                if cfg!(feature = "timer-queue") {
                    quote! {
                    #_priority::Task::#name => {
                        let payload =
                            core::ptr::read(::#name::PAYLOADS.get_unchecked(index as usize));
                        let ss = *::#name::SCHEDULED_TIMES.get_unchecked(index as usize);

                        #name::FREE_QUEUE::_var().split().0.enqueue_unchecked(index);

                        #name::HANDLER(#name::Context::new(ss, payload));
                    }

                    }
                } else {
                    quote! {
                    #_priority::Task::#name => {
                        let payload =
                            core::ptr::read(::#name::PAYLOADS.get_unchecked(index as usize));
                        #name::FREE_QUEUE::_var().split().0.enqueue_unchecked(index);
                        #name::HANDLER(#name::Context::new(payload));
                    }
                    }
                }
            })
            .collect::<Vec<_>>();

        let interrupt = dispatcher.interrupt();
        let export_name = interrupt.to_string();
        let fn_name = Ident::new(&format!("_{}", export_name), Span::call_site());
        root.push(quote! {
            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            #[export_name = #export_name]
            pub unsafe extern "C" fn #fn_name() {
                use ::#k::Resource;

                // NOTE(_var) the dispatcher is the only consumer of this queue
                while let Some((task, index)) =
                    #_priority::READY_QUEUE::_var().split().1.dequeue() {
                    match task {
                        #(#arms,)*
                    }
                }
            }
        })
    }

    /* pre-init */
    // Initialize the slot queues
    let mut pre_init = vec![];
    for (name, task) in &app.tasks {
        let input = &task.input;

        if let Either::Right(instances) = task.interrupt_or_instances {
            pre_init.push(quote! {
                for i in 0..#instances {
                    #name::FREE_QUEUE::_var().enqueue_unchecked(i);
                }
            })
        }
    }

    let prio_bits = quote!(#device::NVIC_PRIO_BITS);

    if needs_tq {
        pre_init.push(quote! {
            // Configure the system timer
            p.SYST.set_clock_source(::#k::_impl::SystClkSource::Core);
            p.SYST.enable_counter();

            // Initialize the timer queue
            core::ptr::write(_tq::TIMER_QUEUE::_var(), ::#k::_impl::TimerQueue::new(p.SYST));
        });
    }

    let core = if cfg!(feature = "timer-queue") {
        quote! {
            ::#k::_impl::Peripherals {
                CBP: p.CBP,
                CPUID: p.CPUID,
                DCB: p.DCB,
                // DWT: p.DWT,
                FPB: p.FPB,
                FPU: p.FPU,
                ITM: p.ITM,
                MPU: p.MPU,
                SCB: &mut p.SCB,
                // SYST: p.SYST,
                TPIU: p.TPIU,
            }
        }
    } else {
        quote! {
            ::#k::_impl::Peripherals {
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
            }
        }
    };

    /* init */
    let res_fields = app
        .init
        .resources
        .iter()
        .map(|r| {
            if app.resources[r].expr.is_some() {
                quote!(pub #r: ::#r)
            } else {
                quote!(pub #r: #k::_impl::Uninit<::#r>)
            }
        })
        .collect::<Vec<_>>();

    let res_exprs = app
        .init
        .resources
        .iter()
        .map(|r| quote!(#r: #r::_new()))
        .collect::<Vec<_>>();

    let tasks_fields = app
        .init
        .schedule_now
        .iter()
        .map(|task| quote!(pub #task: ::_schedule_now::#task))
        .chain(
            app.init
                .schedule_after
                .iter()
                .map(|task| quote!(pub #task: ::_schedule_after::#task)),
        )
        .collect::<Vec<_>>();

    let tasks_exprs = app
        .init
        .schedule_now
        .iter()
        .map(|task| {
            if cfg!(feature = "timer-queue") {
                quote!(#task: ::_schedule_now::#task::new(_bl))
            } else {
                quote!(#task: ::_schedule_now::#task::new())
            }
        })
        .chain(
            app.init
                .schedule_after
                .iter()
                .map(|task| quote!(#task: ::_schedule_after::#task::new(_bl))),
        )
        .collect::<Vec<_>>();

    let late_resources = app
        .resources
        .iter()
        .filter_map(|(name, res)| {
            if res.expr.is_none() && !app.init.resources.contains(name) {
                let ty = &res.ty;
                Some(quote!(pub #name: #ty))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let bl = if cfg!(feature = "timer-queue") {
        Some(quote!(let _bl = ::#k::_impl::Instant(0);))
    } else {
        None
    };
    root.push(quote! {
        #[allow(non_snake_case)]
        pub struct _ZN4init13LateResourcesE {
            #(#late_resources,)*
        }

        mod init {
            #[allow(unused_imports)]
            use ::#k::Resource;

            pub use ::#device::Peripherals as Device;
            pub use ::_ZN4init13LateResourcesE as LateResources;

            #[allow(dead_code)]
            pub struct Context<'a> {
                pub core: ::#k::_impl::Peripherals<'a>,
                pub device: Device,
                pub resources: Resources,
                pub tasks: Tasks,
                pub priority: ::#k::Priority<::#k::_impl::U255>,
            }

            #[allow(unsafe_code)]
            impl<'a> Context<'a> {
                pub unsafe fn new(core: ::#k::_impl::Peripherals<'a>) -> Self {
                    Context {
                        tasks: Tasks::new(),
                        core,
                        device: Device::steal(),
                        resources: Resources::new(),
                        priority: ::#k::Priority::_new(),
                    }
                }
            }

            pub struct Tasks {
                #(#tasks_fields,)*
            }

            #[allow(unsafe_code)]
            impl Tasks {
                unsafe fn new() -> Self {
                    #bl

                    Tasks {
                        #(#tasks_exprs,)*
                    }
                }
            }

            #[allow(non_snake_case)]
            pub struct Resources {
                #(#res_fields,)*
            }

            #[allow(unsafe_code)]
            impl Resources {
                unsafe fn new() -> Self {
                    Resources {
                        #(#res_exprs,)*
                    }
                }
            }
        }
    });

    /* post-init */
    let mut post_init = vec![];

    if needs_tq {
        let priority = ctxt.sys_tick;

        post_init.push(quote! {
            // Set the priority of the SysTick exception
            let priority = ((1 << #prio_bits) - #priority) << (8 - #prio_bits);
            p.SCB.shpr[11].write(priority);
        });
    }

    // Initialize LateResources
    for (name, res) in &app.resources {
        if res.expr.is_none() && !app.init.resources.contains(name) {
            post_init.push(quote! {
                core::ptr::write(_resource::#name::_var(), _lr.#name);
            });
        }
    }

    // Set dispatcher priorities
    for (priority, dispatcher) in &ctxt.dispatchers {
        let interrupt = dispatcher.interrupt();
        post_init.push(quote! {
            let priority = ((1 << #prio_bits) - #priority) << (8 - #prio_bits);
            p.NVIC.set_priority(Interrupt::#interrupt, priority);
        });
    }

    // Set trigger priorities
    for (interrupt, (_, priority)) in &ctxt.triggers {
        post_init.push(quote! {
            let priority = ((1 << #prio_bits) - #priority) << (8 - #prio_bits);
            p.NVIC.set_priority(Interrupt::#interrupt, priority);
        });
    }

    // Enable the dispatcher interrupts
    for dispatcher in ctxt.dispatchers.values() {
        let interrupt = dispatcher.interrupt();
        post_init.push(quote! {
            p.NVIC.enable(Interrupt::#interrupt);
        });
    }

    // Enable triggers
    for interrupt in ctxt.triggers.keys() {
        post_init.push(quote! {
            p.NVIC.enable(Interrupt::#interrupt);
        });
    }

    if app.idle.is_none() {
        post_init.push(quote! {
            // Set SLEEPONEXIT
            p.SCB.scr.modify(|r| r | (1 << 1));
        });
    }

    if needs_tq {
        post_init.push(quote! {
            // Set the system time to zero
            p.DWT.enable_cycle_counter();
            p.DWT.cyccnt.write(0);
        });
    }

    /* idle */
    if let Some(idle) = app.idle.as_ref() {
        let res_fields = idle
            .resources
            .iter()
            .map(|res| {
                if ctxt.ceilings.resources()[res].is_owned() {
                    let ty = &app.resources[res].ty;

                    quote!(pub #res: &'static mut #ty)
                } else {
                    quote!(pub #res: _resource::#res)
                }
            })
            .collect::<Vec<_>>();

        let res_exprs = idle
            .resources
            .iter()
            .map(|res| {
                if ctxt.ceilings.resources()[res].is_owned() {
                    quote!(#res: _resource::#res::_var())
                } else {
                    quote!(#res: _resource::#res::new())
                }
            })
            .collect::<Vec<_>>();

        root.push(quote! {
            mod idle {
                #[allow(unused_imports)]
                use ::#k::Resource;

                #[allow(dead_code)]
                pub struct Context {
                    pub resources: Resources,
                    pub priority: ::#k::Priority<::#k::_impl::U0>,
                }

                #[allow(unsafe_code)]
                impl Context {
                    pub unsafe fn new() -> Self {
                        Context {
                            resources: Resources::new(),
                            priority: ::#k::Priority::_new(),
                        }
                    }
                }

                #[allow(non_snake_case)]
                pub struct Resources {
                    #(#res_fields,)*
                }

                #[allow(unsafe_code)]
                impl Resources {
                    unsafe fn new() -> Self {
                        Resources {
                            #(#res_exprs,)*
                        }
                    }
                }
            }
        });
    }

    /* main */
    let init = &app.init.path;
    let mut main = vec![quote! {
        #[allow(unused_imports)]
        use ::#k::Resource;
        #[allow(unused_imports)]
        use #device::Interrupt;

        let init: fn(init::Context) -> init::LateResources = #init;

        ::#k::_impl::interrupt::disable();

        let mut p = ::#k::_impl::steal();

        #(#pre_init)*

        let _lr = init(init::Context::new(#core));

        #(#post_init)*

        ::#k::_impl::interrupt::enable();
    }];

    if let Some(idle) = app.idle.as_ref().map(|idle| &idle.path) {
        main.push(quote! {
            let idle: fn(idle::Context) -> ! = #idle;
            idle(idle::Context::new())
        });
    } else {
        main.push(quote! {
            loop {
                #k::_impl::asm::wfi();
            }
        });
    }

    root.push(quote! {
        #[allow(unsafe_code)]
        #[allow(unused_mut)]
        #[deny(const_err)]
        #[no_mangle]
        pub unsafe extern "C" fn main() -> ! {
            #(#main)*
        }
    });

    quote! {
        #(#root)*
    }
}
