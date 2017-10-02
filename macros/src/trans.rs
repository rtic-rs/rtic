use quote::{Ident, Tokens};
use syn::{Lit, StrStyle};

use analyze::{Ownership, Ownerships};
use check::{App, Kind};

fn krate() -> Ident {
    Ident::from("rtfm")
}

pub fn app(app: &App, ownerships: &Ownerships) -> Tokens {
    let mut root = vec![];
    let mut main = vec![];

    ::trans::init(app, &mut main, &mut root);
    ::trans::idle(app, ownerships, &mut main, &mut root);
    ::trans::resources(app, ownerships, &mut root);
    ::trans::tasks(app, ownerships, &mut root);

    root.push(quote! {
        #[allow(unsafe_code)]
        fn main() {
            #(#main)*
        }
    });

    quote!(#(#root)*)
}

fn idle(
    app: &App,
    ownerships: &Ownerships,
    main: &mut Vec<Tokens>,
    root: &mut Vec<Tokens>,
) {
    let krate = krate();

    let mut mod_items = vec![];
    let mut tys = vec![];
    let mut exprs = vec![];

    if !app.idle.resources.is_empty() {
        tys.push(quote!(&mut #krate::Threshold));
        exprs.push(quote!(unsafe { &mut #krate::Threshold::new(0) }));
    }

    if !app.idle.resources.is_empty() {
        let device = &app.device;

        let mut needs_reexport = false;
        for name in &app.idle.resources {
            if ownerships[name].is_owned() {
                if app.resources.get(name).is_some() {
                    needs_reexport = true;
                    break;
                }
            }
        }

        let super_ = if needs_reexport {
            None
        } else {
            Some(Ident::new("super"))
        };
        let mut rexprs = vec![];
        let mut rfields = vec![];
        for name in &app.idle.resources {
            if ownerships[name].is_owned() {
                if let Some(resource) = app.resources.get(name) {
                    let ty = &resource.ty;

                    rfields.push(quote! {
                        pub #name: &'static mut #ty,
                    });

                    let _name = Ident::new(format!("_{}", name.as_ref()));
                    rexprs.push(if resource.expr.is_some() {
                        quote! {
                            #name: &mut #super_::#_name,
                        }
                    } else {
                        quote! {
                            #name: #super_::#_name.as_mut(),
                        }
                    });
                } else {
                    rfields.push(quote! {
                        pub #name: &'static mut ::#device::#name,
                    });

                    rexprs.push(quote! {
                        #name: &mut *::#device::#name.get(),
                    });
                }
            } else {
                rfields.push(quote! {
                    pub #name: #super_::_resource::#name,
                });

                rexprs.push(quote! {
                    #name: #super_::_resource::#name::new(),
                });
            }
        }

        if needs_reexport {
            root.push(quote! {
                #[allow(non_camel_case_types)]
                #[allow(non_snake_case)]
                pub struct _idleResources {
                    #(#rfields)*
                }
            });

            mod_items.push(quote! {
                pub use ::_idleResources as Resources;
            });
        } else {
            mod_items.push(quote! {
                #[allow(non_snake_case)]
                pub struct Resources {
                    #(#rfields)*
                }
            });
        }

        mod_items.push(quote! {
            #[allow(unsafe_code)]
            impl Resources {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#rexprs)*
                    }
                }
            }
        });

        tys.push(quote!(idle::Resources));
        exprs.push(quote!(unsafe { idle::Resources::new() }));
    }

    if !mod_items.is_empty() {
        root.push(quote! {
            #[allow(unsafe_code)]
            mod idle {
                #(#mod_items)*
            }
        });
    }

    let idle = &app.idle.path;
    main.push(quote! {
        // type check
        let idle: fn(#(#tys),*) -> ! = #idle;

        idle(#(#exprs),*);
    });
}

fn init(app: &App, main: &mut Vec<Tokens>, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    let mut tys = vec![quote!(#device::Peripherals)];
    let mut exprs = vec![quote!(#device::Peripherals::all())];
    let mut ret = None;
    let mut mod_items = vec![];

    let (init_resources, late_resources): (Vec<_>, Vec<_>) = app.resources.iter()
        .partition(|&(_, res)| res.expr.is_some());

    if !init_resources.is_empty() {
        let mut fields = vec![];
        let mut lifetime = None;
        let mut rexprs = vec![];

        for (name, resource) in init_resources {
            let _name = Ident::new(format!("_{}", name.as_ref()));
            lifetime = Some(quote!('a));

            let ty = &resource.ty;

            fields.push(quote! {
                pub #name: &'a mut #krate::Static<#ty>,
            });

            rexprs.push(quote! {
                #name: ::#krate::Static::ref_mut(&mut ::#_name),
            });
        }

        root.push(quote! {
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            pub struct _initResources<#lifetime> {
                #(#fields)*
            }
        });

        mod_items.push(quote! {
            pub use ::_initResources as Resources;

            #[allow(unsafe_code)]
            impl<#lifetime> Resources<#lifetime> {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#rexprs)*
                    }
                }
            }
        });

        tys.push(quote!(init::Resources));
        exprs.push(quote!(init::Resources::new()));
    }

    // Initialization statements for late resources
    let mut late_resource_init = vec![];

    if !late_resources.is_empty() {
        // `init` must initialize and return resources

        let mut fields = vec![];

        for (name, resource) in late_resources {
            let _name = Ident::new(format!("_{}", name.as_ref()));

            let ty = &resource.ty;

            fields.push(quote! {
                pub #name: #ty,
            });

            late_resource_init.push(quote! {
                #_name = #krate::UntaggedOption { some: _late_resources.#name };
            });
        }

        root.push(quote! {
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            pub struct _initLateResourceValues {
                #(#fields)*
            }
        });

        mod_items.push(quote! {
            pub use ::_initLateResourceValues as LateResourceValues;
        });

        // `init` must return the initialized resources
        ret = Some(quote!( -> ::init::LateResourceValues));
    }

    root.push(quote! {
        #[allow(unsafe_code)]
        mod init {
            pub use ::#device::Peripherals;

            #(#mod_items)*
        }
    });

    let mut exceptions = vec![];
    let mut interrupts = vec![];
    for (name, task) in &app.tasks {
        match task.kind {
            Kind::Exception(ref e) => {
                if exceptions.is_empty() {
                    exceptions.push(quote! {
                        let scb = &*#device::SCB.get();
                    });
                }

                let nr = e.nr();
                let priority = task.priority;
                exceptions.push(quote! {
                    let prio_bits = #device::NVIC_PRIO_BITS;
                    let hw = ((1 << prio_bits) - #priority) << (8 - prio_bits);
                    scb.shpr[#nr - 4].write(hw);
                });
            }
            Kind::Interrupt { enabled } => {
                // Interrupt. These can be enabled / disabled through the NVIC
                if interrupts.is_empty() {
                    interrupts.push(quote! {
                        let nvic = &*#device::NVIC.get();
                    });
                }

                let priority = task.priority;
                interrupts.push(quote! {
                    let prio_bits = #device::NVIC_PRIO_BITS;
                    let hw = ((1 << prio_bits) - #priority) << (8 - prio_bits);
                    nvic.set_priority(#device::Interrupt::#name, hw);
                });

                if enabled {
                    interrupts.push(quote! {
                        nvic.enable(#device::Interrupt::#name);
                    });
                } else {
                    interrupts.push(quote! {
                        nvic.disable(#device::Interrupt::#name);
                    });
                }
            }
        }
    }

    let init = &app.init.path;
    main.push(quote! {
        // type check
        let init: fn(#(#tys,)*) #ret = #init;

        #krate::atomic(unsafe { &mut #krate::Threshold::new(0) }, |_t| unsafe {
            let _late_resources = init(#(#exprs,)*);
            #(#late_resource_init)*

            #(#exceptions)*
            #(#interrupts)*
        });
    });
}

fn resources(app: &App, ownerships: &Ownerships, root: &mut Vec<Tokens>) {
    let krate = krate();
    let device = &app.device;

    let mut items = vec![];
    let mut impls = vec![];
    for (name, ownership) in ownerships {
        let _name = Ident::new(format!("_{}", name.as_ref()));

        if let Some(resource) = app.resources.get(name) {
            // Declare the static that holds the resource
            let expr = &resource.expr;
            let ty = &resource.ty;

            root.push(match *expr {
                Some(ref expr) => quote! {
                    static mut #_name: #ty = #expr;
                },
                None => quote! {
                    // Resource initialized in `init`
                    static mut #_name: #krate::UntaggedOption<#ty> = #krate::UntaggedOption { none: () };
                },
            });
        }

        let mut impl_items = vec![];

        match *ownership {
            Ownership::Owned { .. } => {
                // For owned resources we don't need claim() or borrow()
            }
            Ownership::Shared { ceiling } => {
                if let Some(resource) = app.resources.get(name) {
                    let ty = &resource.ty;
                    let res_rvalue = if resource.expr.is_some() {
                        quote!(#_name)
                    } else {
                        quote!(#_name.some)
                    };

                    impl_items.push(quote! {
                        type Data = #ty;

                        fn borrow<'cs>(
                            &'cs self,
                            t: &'cs #krate::Threshold,
                        ) -> &'cs #krate::Static<#ty> {
                            assert!(t.value() >= #ceiling);

                            unsafe { #krate::Static::ref_(&#res_rvalue) }
                        }

                        fn borrow_mut<'cs>(
                            &'cs mut self,
                            t: &'cs #krate::Threshold,
                        ) -> &'cs mut #krate::Static<#ty> {
                            assert!(t.value() >= #ceiling);

                            unsafe {
                                #krate::Static::ref_mut(&mut #res_rvalue)
                            }
                        }

                        fn claim<R, F>(
                            &self,
                            t: &mut #krate::Threshold,
                            f: F,
                        ) -> R
                        where
                            F: FnOnce(
                                &#krate::Static<#ty>,
                                &mut #krate::Threshold) -> R
                        {
                            unsafe {
                                #krate::claim(
                                    #krate::Static::ref_(&#res_rvalue),
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }

                        fn claim_mut<R, F>(
                            &mut self,
                            t: &mut #krate::Threshold,
                            f: F,
                        ) -> R
                        where
                            F: FnOnce(
                                &mut #krate::Static<#ty>,
                                &mut #krate::Threshold) -> R
                        {
                            unsafe {
                                #krate::claim(
                                    #krate::Static::ref_mut(&mut #res_rvalue),
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }
                    });
                } else {
                    impl_items.push(quote! {
                        type Data = #device::#name;

                        fn borrow<'cs>(
                            &'cs self,
                            t: &'cs #krate::Threshold,
                        ) -> &'cs #krate::Static<#device::#name> {
                            assert!(t.value() >= #ceiling);

                            unsafe {
                                #krate::Static::ref_(&*#device::#name.get())
                            }
                        }

                        fn borrow_mut<'cs>(
                            &'cs mut self,
                            t: &'cs #krate::Threshold,
                        ) -> &'cs mut #krate::Static<#device::#name> {
                            assert!(t.value() >= #ceiling);

                            unsafe {
                                #krate::Static::ref_mut(
                                    &mut *#device::#name.get(),
                                )
                            }
                        }

                        fn claim<R, F>(
                            &self,
                            t: &mut #krate::Threshold,
                            f: F,
                        ) -> R
                        where
                            F: FnOnce(
                                &#krate::Static<#device::#name>,
                                &mut #krate::Threshold) -> R
                        {
                            unsafe {
                                #krate::claim(
                                    #krate::Static::ref_(
                                        &*#device::#name.get(),
                                    ),
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }

                        fn claim_mut<R, F>(
                            &mut self,
                            t: &mut #krate::Threshold,
                            f: F,
                        ) -> R
                        where
                            F: FnOnce(
                                &mut #krate::Static<#device::#name>,
                                &mut #krate::Threshold) -> R
                        {
                            unsafe {
                                #krate::claim(
                                    #krate::Static::ref_mut(
                                        &mut *#device::#name.get(),
                                    ),
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }
                    });
                }

                impls.push(quote! {
                    #[allow(unsafe_code)]
                    unsafe impl #krate::Resource for _resource::#name {
                        #(#impl_items)*
                    }
                });

                items.push(quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #name { _0: () }

                    #[allow(unsafe_code)]
                    impl #name {
                        pub unsafe fn new() -> Self {
                            #name { _0: () }
                        }
                    }
                });
            }
        }
    }

    if !items.is_empty() {
        root.push(quote! {
            #[allow(unsafe_code)]
            mod _resource {
                #(#items)*
            }
        })
    }
    root.push(quote! {
        #(#impls)*
    });
}

fn tasks(app: &App, ownerships: &Ownerships, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    for (name, task) in &app.tasks {
        let mut exprs = vec![];
        let mut fields = vec![];
        let mut items = vec![];

        let mut lifetime = None;
        let mut needs_reexport = false;
        let mut needs_threshold = false;
        let has_resources = !task.resources.is_empty();

        if has_resources {
            needs_threshold = !task.resources.is_empty();

            for name in &task.resources {
                let _name = Ident::new(format!("_{}", name.as_ref()));

                match ownerships[name] {
                    Ownership::Shared { ceiling }
                        if ceiling > task.priority =>
                    {
                        needs_threshold = true;

                        fields.push(quote! {
                            pub #name: ::_resource::#name,
                        });

                        exprs.push(quote! {
                            #name: {
                                ::_resource::#name::new()
                            },
                        });
                    }
                    _ => {
                        lifetime = Some(quote!('a));
                        if let Some(resource) = app.resources.get(name) {
                            needs_reexport = true;
                            let ty = &resource.ty;

                            fields.push(quote! {
                                pub #name: &'a mut ::#krate::Static<#ty>,
                            });

                            exprs.push(if resource.expr.is_some() {
                                quote! {
                                    #name: ::#krate::Static::ref_mut(&mut ::#_name),
                                }
                            } else {
                                quote! {
                                    #name: ::#krate::Static::ref_mut(::#_name.as_mut()),
                                }
                            });
                        } else {
                            fields.push(quote! {
                                pub #name:
                                &'a mut ::#krate::Static<::#device::#name>,
                            });

                            exprs.push(quote! {
                                #name: ::#krate::Static::ref_mut(
                                    &mut *::#device::#name.get(),
                                ),
                            });
                        }
                    }
                }
            }

            if needs_reexport {
                let rname = Ident::new(format!("_{}Resources", name));
                root.push(quote! {
                    #[allow(non_camel_case_types)]
                    #[allow(non_snake_case)]
                    pub struct #rname<#lifetime> {
                        #(#fields)*
                    }
                });

                items.push(quote! {
                    pub use ::#rname as Resources;
                });
            } else {
                items.push(quote! {
                    #[allow(non_snake_case)]
                    pub struct Resources<#lifetime> {
                        #(#fields)*
                    }
                });
            }

            items.push(quote! {
                #[allow(unsafe_code)]
                impl<#lifetime> Resources<#lifetime> {
                    pub unsafe fn new() -> Self {
                        Resources {
                            #(#exprs)*
                        }
                    }
                }
            });
        }

        let mut tys = vec![];
        let mut exprs = vec![];

        let priority = task.priority;
        if needs_threshold {
            tys.push(quote!(&mut #krate::Threshold));
            exprs.push(quote! {
                &mut if #priority == 1 << #device::NVIC_PRIO_BITS {
                    #krate::Threshold::new(::core::u8::MAX)
                } else {
                    #krate::Threshold::new(#priority)
                }
            });
        }

        if has_resources {
            tys.push(quote!(#name::Resources));
            exprs.push(quote!(#name::Resources::new()));
        }

        let path = &task.path;
        let _name = Ident::new(format!("_{}", name));
        let export_name =
            Lit::Str(name.as_ref().to_owned(), StrStyle::Cooked);
        root.push(quote! {
            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            #[export_name = #export_name]
            pub unsafe extern "C" fn #_name() {
                let f: fn(#(#tys,)*) = #path;

                f(#(#exprs,)*)
            }
        });

        root.push(quote!{
            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            mod #name {
                #[allow(dead_code)]
                #[deny(const_err)]
                const CHECK_PRIORITY: (u8, u8) = (
                    #priority - 1,
                    (1 << ::#device::NVIC_PRIO_BITS) - #priority,
                );

                #(#items)*
            }
        });
    }
}
