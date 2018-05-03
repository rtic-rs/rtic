use quote::Tokens;

use either::Either;
use syn::Ident;
use syntax::check::App;

use analyze::Context;

pub fn app(ctxt: &Context, app: &App) -> Tokens {
    let mut root = vec![];
    let krate = Ident::from("cortex_m_rtfm");
    let device = &app.device;
    let hidden = Ident::from("__hidden");

    let needs_tq = !ctxt.async_after.is_empty();

    /* root */
    // NOTE we can't use paths like `#krate::foo` in the root because there's no guarantee that the
    // user has not renamed `cortex_m_rtfm` (e.g. `extern crate cortex_m_rtfm as rtfm`) so instead
    // we add this `#hidden` module and use `#hidden::#krate::foo` in the root.
    root.push(quote! {
        mod #hidden {
            pub extern crate #krate;
        }
    });

    /* Resources */
    let mut resources = vec![];
    for (name, resource) in &app.resources {
        let ty = &resource.ty;
        let expr = resource
            .expr
            .as_ref()
            .map(|e| quote!(#e))
            .unwrap_or_else(|| quote!(unsafe { #hidden::#krate::uninitialized() }));

        let ceiling = Ident::from(format!(
            "U{}",
            ctxt.ceilings
                .resources()
                .get(name)
                .cloned()
                .map(u8::from)
                .unwrap_or(0) // 0 = resource owned by `init`
        ));
        root.push(quote! {
            #[allow(unsafe_code)]
            unsafe impl #hidden::#krate::Resource for __resource::#name {
                const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                type Ceiling = #hidden::#krate::#ceiling;
                type Data = #ty;

                unsafe fn get() -> &'static mut Self::Data {
                    static mut #name: #ty = #expr;

                    &mut #name
                }
            }
        });

        resources.push(quote! {
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
        mod __resource {
            extern crate #krate;

            #[allow(unused_imports)]
            use core::marker::PhantomData;

            #(#resources)*
        }
    });

    /* Tasks */
    for (name, task) in &app.tasks {
        let path = &task.path;
        let input = &task.input;

        let lifetime = if task.resources
            .iter()
            .any(|res| ctxt.ceilings.resources()[res].is_owned())
        {
            Some(quote!('a))
        } else {
            None
        };

        let __context = Ident::from(format!(
            "_ZN{}{}7ContextE",
            name.as_ref().as_bytes().len(),
            name
        ));

        let mut mod_ = vec![];

        // NOTE some stuff has to go in the root because `#input` is not guaranteed to be a
        // primitive type and there's no way to import that type into a module (we don't know its
        // full path). So instead we just assume that `#input` has been imported in the root; this
        // forces us to put anything that refers to `#input` in the root.
        if cfg!(feature = "timer-queue") {
            root.push(quote! {
                pub struct #__context<#lifetime> {
                    pub async: #name::Async,
                    pub baseline: u32,
                    pub input: #input,
                    pub resources: #name::Resources<#lifetime>,
                    pub threshold: #hidden::#krate::Threshold<#name::Priority>,
                }

                #[allow(unsafe_code)]
                impl<#lifetime> #__context<#lifetime> {
                    pub unsafe fn new(bl: #hidden::#krate::Instant, payload: #input) -> Self {
                        #__context {
                            async: #name::Async::new(bl),
                            baseline: bl.into(),
                            input: payload,
                            resources: #name::Resources::new(),
                            threshold: #hidden::#krate::Threshold::new(),
                        }
                    }
                }
            });
        } else {
            root.push(quote! {
                pub struct #__context<#lifetime> {
                    pub async: #name::Async,
                    pub input: #input,
                    pub resources: #name::Resources<#lifetime>,
                    pub threshold: #hidden::#krate::Threshold<#name::Priority>,
                }

                #[allow(unsafe_code)]
                impl<#lifetime> #__context<#lifetime> {
                    pub unsafe fn new(payload: #input) -> Self {
                        #__context {
                            async: #name::Async::new(),
                            input: payload,
                            resources: #name::Resources::new(),
                            threshold: #hidden::#krate::Threshold::new(),
                        }
                    }
                }
            });
        }

        let res_fields = task.resources
            .iter()
            .map(|res| {
                if ctxt.ceilings.resources()[res].is_owned() {
                    let ty = &app.resources[res].ty;
                    quote!(pub #res: &'a mut #ty)
                } else {
                    quote!(pub #res: super::__resource::#res)
                }
            })
            .collect::<Vec<_>>();

        let res_exprs = task.resources.iter().map(|res| {
            if ctxt.ceilings.resources()[res].is_owned() {
                quote!(#res: super::__resource::#res::get())
            } else {
                quote!(#res: super::__resource::#res::new())
            }
        });

        let async_fields = task.async
            .iter()
            .map(|task| quote!(pub #task: ::__async::#task))
            .chain(
                task.async_after
                    .iter()
                    .map(|task| quote!(pub #task: ::__async_after::#task)),
            )
            .collect::<Vec<_>>();

        let async_exprs = task.async
            .iter()
            .map(|task| {
                if cfg!(feature = "timer-queue") {
                    quote!(#task: ::__async::#task::new(_bl))
                } else {
                    quote!(#task: ::__async::#task::new())
                }
            })
            .chain(
                task.async_after
                    .iter()
                    .map(|task| quote!(#task: ::__async_after::#task::new(_bl))),
            )
            .collect::<Vec<_>>();

        let priority = Ident::from(format!("U{}", task.priority));
        mod_.push(quote! {
            extern crate #krate;

            #[allow(unused_imports)]
            use self::#krate::Resource;

            pub const HANDLER: fn(Context) = ::#path;

            // The priority at this task is dispatched at
            pub type Priority = #krate::#priority;

            pub use super::#__context as Context;

            pub struct Async {
                #(#async_fields,)*
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
                impl Async {
                    pub unsafe fn new(_bl: #krate::Instant) -> Self {
                        Async {
                            #(#async_exprs,)*
                        }
                    }
                }

            });
        } else {
            mod_.push(quote! {
                #[allow(unsafe_code)]
                impl Async {
                    pub unsafe fn new() -> Self {
                        Async {
                            #(#async_exprs,)*
                        }
                    }
                }

            });
        }

        match task.interrupt_or_capacity {
            Either::Left(interrupt) => {
                let export_name = interrupt.as_ref();
                let fn_name = Ident::from(format!("__{}", interrupt));

                let bl = if cfg!(feature = "timer-queue") {
                    Some(quote!(#hidden::#krate::Instant::now(),))
                } else {
                    None
                };
                root.push(quote! {
                    #[allow(non_snake_case)]
                    #[allow(unsafe_code)]
                    #[export_name = #export_name]
                    pub unsafe extern "C" fn #fn_name() {
                        let _ = #device::Interrupt::#interrupt; // verify that the interrupt exists
                        #name::HANDLER(#name::Context::new(#bl ()))
                    }
                });
            }
            Either::Right(capacity) => {
                let capacity = Ident::from(format!("U{}", capacity));

                root.push(quote! {
                    #[allow(unsafe_code)]
                    unsafe impl #hidden::#krate::Resource for #name::SQ {
                        const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                        type Ceiling = #name::Ceiling;
                        type Data = #hidden::#krate::SlotQueue<#input, #hidden::#krate::#capacity>;

                        unsafe fn get() -> &'static mut Self::Data {
                            static mut SQ:
                                #hidden::#krate::SlotQueue<#input, #hidden::#krate::#capacity> =
                                #hidden::#krate::SlotQueue::u8();

                            &mut SQ
                        }
                    }

                });

                let ceiling = Ident::from(format!(
                    "U{}",
                    ctxt.ceilings.slot_queues().get(name).cloned() // 0 = owned by init
                        .unwrap_or(0)
                ));
                mod_.push(quote! {
                    pub struct SQ { _0: () }

                    #[allow(unsafe_code)]
                    impl SQ {
                        pub unsafe fn new() -> Self {
                            SQ { _0: () }
                        }
                    }

                    // Ceiling of the `SQ` resource
                    pub type Ceiling = #krate::#ceiling;
                });
            }
        }

        root.push(quote! {
            mod #name {
                #(#mod_)*
            }
        });
    }

    /* Async */
    let async = ctxt.async
        .iter()
        .map(|name| {
            let task = &app.tasks[name];
            let priority = task.priority;
            let __priority = Ident::from(format!("__{}", priority));
            let interrupt = ctxt.dispatchers[&priority].interrupt();
            let ty = &task.input;

            let sqc = Ident::from(format!(
                "U{}",
                ctxt.ceilings.slot_queues().get(name).cloned() // 0 = owned by init
                    .unwrap_or(0)
            ));
            let qc = Ident::from(format!("U{}", ctxt.ceilings.dispatch_queues()[&priority]));

            if cfg!(feature = "timer-queue") {
                quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #name { baseline: #krate::Instant }

                    #[allow(unsafe_code)]
                    impl #name {
                        pub unsafe fn new(bl: #krate::Instant) -> Self {
                            #name { baseline: bl }
                        }

                        // XXX or take `self`?
                        #[inline]
                        pub fn post<P>(
                            &self,
                            t: &mut #krate::Threshold<P>,
                            payload: #ty,
                        ) -> Result<(), #ty>
                        where
                            P: #krate::Unsigned +
                                #krate::Max<#krate::#sqc> +
                                #krate::Max<#krate::#qc>,
                            #krate::Maximum<P, #krate::#sqc>: #krate::Unsigned,
                            #krate::Maximum<P, #krate::#qc>: #krate::Unsigned,
                        {
                            unsafe {
                                let slot = ::#name::SQ::new().claim_mut(t, |sq, _| sq.dequeue());
                                if let Some(slot) = slot {
                                    let tp = slot
                                        .write(self.baseline, payload)
                                        .tag(::#__priority::Task::#name);

                                    ::#__priority::Q::new().claim_mut(t, |q, _| {
                                        q.split().0.enqueue_unchecked(tp);
                                    });

                                    #krate::set_pending(#device::Interrupt::#interrupt);

                                    Ok(())
                                } else {
                                    Err(payload)
                                }
                            }
                        }
                    }
                }
            } else {
                quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #name {}

                    #[allow(unsafe_code)]
                    impl #name {
                        pub unsafe fn new() -> Self {
                            #name {}
                        }

                        // XXX or take `self`?
                        #[inline]
                        pub fn post<P>(
                            &self,
                            t: &mut #krate::Threshold<P>,
                            payload: #ty,
                        ) -> Result<(), #ty>
                        where
                            P: #krate::Unsigned +
                                #krate::Max<#krate::#sqc> +
                                #krate::Max<#krate::#qc>,
                            #krate::Maximum<P, #krate::#sqc>: #krate::Unsigned,
                            #krate::Maximum<P, #krate::#qc>: #krate::Unsigned,
                        {
                            unsafe {
                                if let Some(slot) =
                                    ::#name::SQ::new().claim_mut(t, |sq, _| sq.dequeue()) {
                                    let tp = slot
                                        .write(payload)
                                        .tag(::#__priority::Task::#name);

                                    ::#__priority::Q::new().claim_mut(t, |q, _| {
                                        q.split().0.enqueue_unchecked(tp);
                                    });

                                        #krate::set_pending(#device::Interrupt::#interrupt);

                                    Ok(())
                                } else {
                                    Err(payload)
                                }
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    root.push(quote! {
        mod __async {
            extern crate #krate;

            #[allow(unused_imports)]
            use self::#krate::Resource;

            #(#async)*
        }
    });

    /* Async (+after) */
    let async_after = ctxt.async_after
        .iter()
        .map(|name| {
            let task = &app.tasks[name];
            let ty = &task.input;

            let sqc = Ident::from(format!("U{}", ctxt.ceilings.slot_queues()[name]));
            let tqc = Ident::from(format!("U{}", ctxt.ceilings.timer_queue()));

            quote! {
                #[allow(non_camel_case_types)]
                pub struct #name { baseline: #krate::Instant }

                #[allow(unsafe_code)]
                impl #name {
                    pub unsafe fn new(bl: #krate::Instant) -> Self {
                        #name { baseline: bl }
                    }

                    // XXX or take `self`?
                    #[inline]
                    pub fn post<P>(
                        &self,
                        t: &mut #krate::Threshold<P>,
                        after: u32,
                        payload: #ty,
                    ) -> Result<(), #ty>
                    where
                        P: #krate::Unsigned +
                            #krate::Max<#krate::#sqc> +
                            #krate::Max<#krate::#tqc>,
                        #krate::Maximum<P, #krate::#sqc>: #krate::Unsigned,
                        #krate::Maximum<P, #krate::#tqc>: #krate::Unsigned,
                    {
                        unsafe {
                            if let Some(slot) =
                                ::#name::SQ::new().claim_mut(t, |sq, _| sq.dequeue()) {
                                let bl = self.baseline + after;
                                let tp = slot
                                    .write(bl, payload)
                                    .tag(::__tq::Task::#name);

                                ::__tq::TQ::new().claim_mut(t, |tq, _| tq.enqueue(bl, tp));

                                Ok(())
                            } else {
                                Err(payload)
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    root.push(quote! {
        mod __async_after {
            extern crate #krate;

            #[allow(unused_imports)]
            use self::#krate::Resource;

            #(#async_after)*
        }
    });

    /* Timer queue */
    if needs_tq {
        let capacity = Ident::from(format!("U{}", ctxt.timer_queue.capacity()));
        let tasks = ctxt.timer_queue.tasks().keys();
        let arms = ctxt.timer_queue
            .tasks()
            .iter()
            .map(|(name, priority)| {
                let __priority = Ident::from(format!("__{}", priority));
                let interrupt = ctxt.dispatchers[priority].interrupt();

                quote! {
                    __tq::Task::#name => {
                        #__priority::Q::new().claim_mut(t, |q, _| {
                            q.split().0.enqueue_unchecked(tp.retag(#__priority::Task::#name))
                        });
                        #hidden::#krate::set_pending(#device::Interrupt::#interrupt);
                    }
                }
            })
            .collect::<Vec<_>>();

        let ceiling = Ident::from(format!("U{}", ctxt.ceilings.timer_queue()));
        let priority = Ident::from(format!("U{}", ctxt.sys_tick));
        root.push(quote! {
            mod __tq {
                extern crate #krate;

                pub struct TQ { _0: () }

                #[allow(unsafe_code)]
                impl TQ {
                    pub unsafe fn new() -> Self {
                        TQ { _0: () }
                    }
                }

                #[allow(unsafe_code)]
                unsafe impl #krate::Resource for TQ {
                    const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                    type Ceiling = #krate::#ceiling;
                    type Data = #krate::TimerQueue<Task, #krate::#capacity>;

                    unsafe fn get() -> &'static mut Self::Data {
                        static mut TQ: #krate::TimerQueue<Task, #krate::#capacity> =
                            unsafe { #krate::uninitialized() };

                        &mut TQ
                    }
                }

                // SysTick priority
                pub type Priority = #krate::#priority;

                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                pub enum Task { #(#tasks,)* }
            }

            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            #[export_name = "SYS_TICK"]
            pub unsafe extern "C" fn __SYS_TICK() {
                use #hidden::#krate::Resource;

                #hidden::#krate::dispatch(
                    &mut #hidden::#krate::Threshold::<__tq::Priority>::new(),
                    &mut __tq::TQ::new(),
                    |t, tp| {
                        match tp.tag() {
                            #(#arms,)*
                        }
                    })
            }
        });
    }

    /* Dispatchers */
    for (priority, dispatcher) in &ctxt.dispatchers {
        let __priority = Ident::from(format!("__{}", priority));
        let capacity = Ident::from(format!("U{}", dispatcher.capacity()));
        let tasks = dispatcher.tasks();
        let ceiling = Ident::from(format!("U{}", ctxt.ceilings.dispatch_queues()[priority]));

        root.push(quote! {
            mod #__priority {
                extern crate #krate;

                pub struct Q { _0: () }

                #[allow(unsafe_code)]
                #[allow(dead_code)]
                impl Q {
                    pub unsafe fn new() -> Self {
                        Q { _0: () }
                    }
                }

                #[allow(unsafe_code)]
                unsafe impl #krate::Resource for Q {
                    const NVIC_PRIO_BITS: u8 = ::#device::NVIC_PRIO_BITS;
                    type Ceiling = #krate::#ceiling;
                    type Data = #krate::PayloadQueue<Task, #krate::#capacity>;

                    unsafe fn get() -> &'static mut Self::Data {
                        static mut Q: #krate::PayloadQueue<Task, #krate::#capacity> =
                            #krate::PayloadQueue::u8();

                        &mut Q
                    }
                }

                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                pub enum Task { #(#tasks,)* }
            }
        });

        let arms = dispatcher
            .tasks()
            .iter()
            .map(|name| {
                // NOTE(get) this is the only `Slot` producer because a task can only be
                // dispatched at one priority
                if cfg!(feature = "timer-queue") {
                    quote! {
                    #__priority::Task::#name => {
                        let (bl, payload, slot) = payload.coerce().read();
                        // priority
                        #name::SQ::get().split().0.enqueue_unchecked(slot);
                        #name::HANDLER(#name::Context::new(bl, payload));
                    }

                    }
                } else {
                    quote! {
                    #__priority::Task::#name => {
                        let (payload, slot) = payload.coerce().read();
                        // priority
                        #name::SQ::get().split().0.enqueue_unchecked(slot);
                        #name::HANDLER(#name::Context::new(payload));
                    }
                    }
                }
            })
            .collect::<Vec<_>>();

        let interrupt = dispatcher.interrupt();
        let export_name = interrupt.as_ref();
        let fn_name = Ident::from(format!("__{}", export_name));
        root.push(quote! {
            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            #[export_name = #export_name]
            pub unsafe extern "C" fn #fn_name() {
                use #hidden::#krate::Resource;

                // NOTE(get) the dispatcher is the only consumer of this queue
                while let Some(payload) = #__priority::Q::get().split().1.dequeue() {
                    match payload.tag() {
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

        if let Either::Right(capacity) = task.interrupt_or_capacity {
            let capacity = capacity as usize;

            pre_init.push(quote! {
                {
                    static mut N: [#hidden::#krate::Node<#input>; #capacity] =
                        unsafe { #hidden::#krate::uninitialized() };

                    for node in N.iter_mut() {
                        #name::SQ::get().enqueue_unchecked(node.into());
                    }
                }
            })
        }
    }

    let prio_bits = quote!(#device::NVIC_PRIO_BITS);
    if needs_tq {
        let priority = ctxt.sys_tick;

        pre_init.push(quote! {
            // Configure the system timer
            _syst.set_clock_source(#hidden::#krate::SystClkSource::Core);
            _syst.enable_counter();

            // Set the priority of the SysTick exception
            let priority = ((1 << #prio_bits) - #priority) << (8 - #prio_bits);
            core.SCB.shpr[11].write(priority);

            // Initialize the timer queue
            core::ptr::write(__tq::TQ::get(), #hidden::#krate::TimerQueue::new(_syst));
        });
    }

    /* init */
    let res_fields = app.init
        .resources
        .iter()
        .map(|r| {
            let ty = &app.resources[r].ty;
            quote!(#r: &'static mut #ty)
        })
        .collect::<Vec<_>>();

    let res_exprs = app.init
        .resources
        .iter()
        .map(|r| quote!(#r: __resource::#r::get()))
        .collect::<Vec<_>>();

    let async_fields = app.init
        .async
        .iter()
        .map(|task| quote!(pub #task: ::__async::#task))
        .chain(
            app.init
                .async_after
                .iter()
                .map(|task| quote!(pub #task: ::__async_after::#task)),
        )
        .collect::<Vec<_>>();

    let async_exprs = app.init
        .async
        .iter()
        .map(|task| {
            if cfg!(feature = "timer-queue") {
                quote!(#task: ::__async::#task::new(_bl))
            } else {
                quote!(#task: ::__async::#task::new())
            }
        })
        .chain(
            app.init
                .async_after
                .iter()
                .map(|task| quote!(#task: ::__async_after::#task::new(_bl))),
        )
        .collect::<Vec<_>>();

    let late_resources = app.resources
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

    let bl = if cfg!(feature = "timer-queue") {
        Some(quote!(let _bl = #krate::Instant::new(0);))
    } else {
        None
    };
    let baseline_field = if cfg!(feature = "timer-queue") {
        Some(quote!(pub baseline: u32,))
    } else {
        None
    };
    let baseline_expr = if cfg!(feature = "timer-queue") {
        Some(quote!(baseline: 0,))
    } else {
        None
    };
    root.push(quote! {
        #[allow(non_snake_case)]
        pub struct _ZN4init13LateResourcesE {
            #(#late_resources,)*
        }

        mod init {
            extern crate #krate;

            #[allow(unused_imports)]
            use self::#krate::Resource;

            pub use ::#device::Peripherals as Device;
            pub use ::_ZN4init13LateResourcesE as LateResources;

            pub struct Context {
                pub async: Async,
                #baseline_field
                pub core: #krate::Core,
                pub device: Device,
                pub resources: Resources,
                pub threshold: #krate::Threshold<#krate::U255>,
            }

            #[allow(unsafe_code)]
            impl Context {
                pub unsafe fn new(core: #krate::Core) -> Self {
                    Context {
                        async: Async::new(),
                        #baseline_expr
                        core,
                        device: Device::steal(),
                        resources: Resources::new(),
                        threshold: #krate::Threshold::new(),
                    }
                }
            }

            pub struct Async {
                #(#async_fields,)*
            }

            #[allow(unsafe_code)]
            impl Async {
                unsafe fn new() -> Self {
                    #bl

                    Async {
                        #(#async_exprs,)*
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

    // Initialize LateResources
    for (name, res) in &app.resources {
        if res.expr.is_none() {
            post_init.push(quote! {
                core::ptr::write(__resource::#name::get(), _lr.#name);
            });
        }
    }

    // Set dispatcher priorities
    for (priority, dispatcher) in &ctxt.dispatchers {
        let interrupt = dispatcher.interrupt();
        post_init.push(quote! {
            let priority = ((1 << #prio_bits) - #priority) << (8 - #prio_bits);
            _nvic.set_priority(#device::Interrupt::#interrupt, priority);
        });
    }

    // Set trigger priorities
    for (interrupt, (_, priority)) in &ctxt.triggers {
        post_init.push(quote! {
            let priority = ((1 << #prio_bits) - #priority) << (8 - #prio_bits);
            _nvic.set_priority(#device::Interrupt::#interrupt, priority);
        });
    }

    // Enable the dispatcher interrupts
    for dispatcher in ctxt.dispatchers.values() {
        let interrupt = dispatcher.interrupt();
        post_init.push(quote! {
            _nvic.enable(#device::Interrupt::#interrupt);
        });
    }

    // Enable triggers
    for interrupt in ctxt.triggers.keys() {
        post_init.push(quote! {
            _nvic.enable(#device::Interrupt::#interrupt);
        });
    }

    /* idle */
    let res_fields = app.idle
        .resources
        .iter()
        .map(|res| {
            if ctxt.ceilings.resources()[res].is_owned() {
                let ty = &app.resources[res].ty;

                quote!(pub #res: &'static mut #ty)
            } else {
                quote!(pub #res: __resource::#res)
            }
        })
        .collect::<Vec<_>>();

    let res_exprs = app.idle
        .resources
        .iter()
        .map(|res| {
            if ctxt.ceilings.resources()[res].is_owned() {
                quote!(#res: __resource::#res::get())
            } else {
                quote!(#res: __resource::#res::new())
            }
        })
        .collect::<Vec<_>>();

    root.push(quote! {
        mod idle {
            extern crate #krate;

            #[allow(unused_imports)]
            use self::#krate::Resource;

            pub struct Context {
                pub resources: Resources,
                pub threshold: #krate::Threshold<#krate::U0>,
            }

            #[allow(unsafe_code)]
            impl Context {
                pub unsafe fn new() -> Self {
                    Context {
                        resources: Resources::new(),
                        threshold: #krate::Threshold::new(),
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

    /* main */
    let idle = &app.idle.path;
    let init = &app.init.path;
    root.push(quote! {
        #[allow(unsafe_code)]
        #[deny(const_err)]
        fn main() {
            #[allow(unused_imports)]
            use #hidden::#krate::Resource;

            #[allow(unused_mut)]
            unsafe {
                let init: fn(init::Context) -> init::LateResources = #init;
                let idle: fn(idle::Context) -> ! = #idle;

                #hidden::#krate::interrupt::disable();

                let (mut core, mut dwt, mut _nvic, mut _syst) = #hidden::#krate::Core::steal();

                #(#pre_init)*

                let _lr = init(init::Context::new(core));

                #(#post_init)*

                // Set the system baseline to zero
                dwt.enable_cycle_counter();
                dwt.cyccnt.write(0);

                #hidden::#krate::interrupt::enable();

                idle(idle::Context::new())
            }
        }
    });

    quote! {
        #(#root)*
    }
}
