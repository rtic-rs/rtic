use proc_macro2::Span;
use quote::Tokens;
use syn::{Ident, LitStr};

use analyze::Ownerships;
use check::{App, Kind};

fn krate() -> Ident {
    Ident::from("rtfm")
}

pub fn app(app: &App, ownerships: &Ownerships) -> Tokens {
    let mut root = vec![];
    let mut main = vec![quote!(#![allow(path_statements)])];

    ::trans::tasks(app, ownerships, &mut root, &mut main);
    ::trans::init(app, &mut main, &mut root);
    ::trans::idle(app, ownerships, &mut main, &mut root);
    ::trans::resources(app, ownerships, &mut root);

    root.push(quote! {
        #[allow(unsafe_code)]
        fn main() {
            #(#main)*
        }
    });

    quote!(#(#root)*)
}

fn idle(app: &App, ownerships: &Ownerships, main: &mut Vec<Tokens>, root: &mut Vec<Tokens>) {
    let krate = krate();

    let mut mod_items = vec![];
    let mut tys = vec![];
    let mut exprs = vec![];

    if !app.idle.resources.is_empty() {
        tys.push(quote!(&mut #krate::Threshold));
        exprs.push(quote!(unsafe { &mut #krate::Threshold::new(0) }));
    }

    if !app.idle.resources.is_empty() {
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
            Some(Ident::from("super"))
        };
        let mut rexprs = vec![];
        let mut rfields = vec![];
        for name in &app.idle.resources {
            if ownerships[name].is_owned() {
                let resource = app.resources.get(name).expect(&format!(
                    "BUG: resource {} assigned to `idle` has no definition",
                    name
                ));
                let ty = &resource.ty;

                rfields.push(quote! {
                    pub #name: &'static mut #ty,
                });

                let _name = Ident::from(format!("_{}", name.as_ref()));
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
                    pub #name: ::idle::#name,
                });

                rexprs.push(quote! {
                    #name: ::idle::#name { _0: core::marker::PhantomData },
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

    let device = &app.device;
    for name in &app.idle.resources {
        let ceiling = ownerships[name].ceiling();

        // owned resource
        if ceiling == 0 {
            continue;
        }

        let _name = Ident::from(format!("_{}", name.as_ref()));
        let resource = app.resources
            .get(name)
            .expect(&format!("BUG: resource {} has no definition", name));

        let ty = &resource.ty;
        let _static = if resource.expr.is_some() {
            quote!(#_name)
        } else {
            quote!(#_name.some)
        };

        mod_items.push(quote! {
            #[allow(non_camel_case_types)]
            pub struct #name { _0: core::marker::PhantomData<*const ()> }
        });

        root.push(quote! {
            #[allow(unsafe_code)]
            unsafe impl #krate::Resource for idle::#name {
                type Data = #ty;

                fn borrow<'cs>(&'cs self, t: &'cs Threshold) -> &'cs Self::Data {
                    assert!(t.value() >= #ceiling);

                    unsafe { &#_static }
                }

                fn borrow_mut<'cs>(
                    &'cs mut self,
                    t: &'cs Threshold,
                ) -> &'cs mut Self::Data {
                    assert!(t.value() >= #ceiling);

                    unsafe { &mut #_static }
                }

                fn claim<R, F>(&self, t: &mut Threshold, f: F) -> R
                where
                    F: FnOnce(&Self::Data, &mut Threshold) -> R
                {
                    unsafe {
                        #krate::claim(
                            &#_static,
                            #ceiling,
                            #device::NVIC_PRIO_BITS,
                            t,
                            f,
                        )
                    }
                }

                fn claim_mut<R, F>(&mut self, t: &mut Threshold, f: F) -> R
                where
                    F: FnOnce(&mut Self::Data, &mut Threshold) -> R
                {
                    unsafe {
                        #krate::claim(
                            &mut #_static,
                            #ceiling,
                            #device::NVIC_PRIO_BITS,
                            t,
                            f,
                        )
                    }
                }
            }
        });
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

    let mut tys = vec![quote!(init::Peripherals)];
    let mut exprs = vec![
        quote!{
            init::Peripherals {
                core: ::#device::CorePeripherals::steal(),
                device: ::#device::Peripherals::steal(),
            }
        },
    ];
    let mut ret = None;
    let mut mod_items = vec![];

    let (init_resources, late_resources): (Vec<_>, Vec<_>) = app.resources
        .iter()
        .partition(|&(_, res)| res.expr.is_some());

    if !init_resources.is_empty() {
        let mut fields = vec![];
        let mut lifetime = None;
        let mut rexprs = vec![];

        for (name, resource) in init_resources {
            let ty = &resource.ty;

            if app.init.resources.contains(name) {
                fields.push(quote! {
                    pub #name: &'static mut #ty,
                });

                let expr = &resource.expr;
                rexprs.push(quote!(#name: {
                    static mut #name: #ty = #expr;
                    &mut #name
                },));
            } else {
                let _name = Ident::from(format!("_{}", name.as_ref()));
                lifetime = Some(quote!('a));

                fields.push(quote! {
                    pub #name: &'a mut #ty,
                });

                rexprs.push(quote! {
                    #name: &mut ::#_name,
                });
            }
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
            let _name = Ident::from(format!("_{}", name.as_ref()));

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
            pub struct _initLateResources {
                #(#fields)*
            }
        });

        mod_items.push(quote! {
            pub use ::_initLateResources as LateResources;
        });

        // `init` must return the initialized resources
        ret = Some(quote!( -> ::init::LateResources));
    }

    root.push(quote! {
        #[allow(unsafe_code)]
        mod init {
            pub struct Peripherals {
                pub core: ::#device::CorePeripherals,
                pub device: ::#device::Peripherals,
            }

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
                        let scb = &*#device::SCB::ptr();
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
                // Interrupt. These are enabled / disabled through the NVIC
                if interrupts.is_empty() {
                    interrupts.push(quote! {
                        use #device::Interrupt;

                        let mut nvic: #device::NVIC = core::mem::transmute(());
                    });
                }

                let priority = task.priority;
                interrupts.push(quote! {
                    let prio_bits = #device::NVIC_PRIO_BITS;
                    let hw = ((1 << prio_bits) - #priority) << (8 - prio_bits);
                    nvic.set_priority(Interrupt::#name, hw);
                });

                if enabled {
                    interrupts.push(quote! {
                        nvic.enable(Interrupt::#name);
                    });
                } else {
                    interrupts.push(quote! {
                        nvic.disable(Interrupt::#name);
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

    for name in ownerships.keys() {
        let _name = Ident::from(format!("_{}", name.as_ref()));

        // Declare the static that holds the resource
        let resource = app.resources
            .get(name)
            .expect(&format!("BUG: resource {} has no definition", name));

        let expr = &resource.expr;
        let ty = &resource.ty;

        root.push(match *expr {
            Some(ref expr) => quote! {
                static mut #_name: #ty = #expr;
            },
            None => quote! {
                // Resource initialized in `init`
                static mut #_name: #krate::UntaggedOption<#ty> =
                    #krate::UntaggedOption { none: () };
            },
        });
    }
}

fn tasks(app: &App, ownerships: &Ownerships, root: &mut Vec<Tokens>, main: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    for (tname, task) in &app.tasks {
        let mut exprs = vec![];
        let mut fields = vec![];
        let mut items = vec![];

        let has_resources = !task.resources.is_empty();

        if has_resources {
            for rname in &task.resources {
                let ceiling = ownerships[rname].ceiling();
                let _rname = Ident::from(format!("_{}", rname.as_ref()));
                let resource = app.resources
                    .get(rname)
                    .expect(&format!("BUG: resource {} has no definition", rname));

                let ty = &resource.ty;
                let _static = if resource.expr.is_some() {
                    quote!(#_rname)
                } else {
                    quote!(#_rname.some)
                };

                items.push(quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #rname { _0: PhantomData<*const ()> }
                });

                root.push(quote! {
                    #[allow(unsafe_code)]
                    unsafe impl #krate::Resource for #tname::#rname {
                        type Data = #ty;

                        fn borrow<'cs>(&'cs self, t: &'cs Threshold) -> &'cs Self::Data {
                            assert!(t.value() >= #ceiling);

                            unsafe { &#_static }
                        }

                        fn borrow_mut<'cs>(
                            &'cs mut self,
                            t: &'cs Threshold,
                        ) -> &'cs mut Self::Data {
                            assert!(t.value() >= #ceiling);

                            unsafe { &mut #_static }
                        }

                        fn claim<R, F>(&self, t: &mut Threshold, f: F) -> R
                        where
                            F: FnOnce(&Self::Data, &mut Threshold) -> R
                        {
                            unsafe {
                                #krate::claim(
                                    &#_static,
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }

                        fn claim_mut<R, F>(&mut self, t: &mut Threshold, f: F) -> R
                        where
                            F: FnOnce(&mut Self::Data, &mut Threshold) -> R
                        {
                            unsafe {
                                #krate::claim(
                                    &mut #_static,
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }
                    }
                });

                if ceiling <= task.priority {
                    root.push(quote! {
                        #[allow(unsafe_code)]
                        impl core::ops::Deref for #tname::#rname {
                            type Target = #ty;

                            fn deref(&self) -> &Self::Target {
                                unsafe { &#_static }
                            }
                        }

                        #[allow(unsafe_code)]
                        impl core::ops::DerefMut for #tname::#rname {
                            fn deref_mut(&mut self) -> &mut Self::Target {
                                unsafe { &mut #_static }
                            }
                        }
                    })
                }

                fields.push(quote! {
                    pub #rname: #rname,
                });

                exprs.push(quote! {
                    #rname: #rname { _0: PhantomData },
                });
            }

            items.push(quote! {
                #[allow(non_snake_case)]
                pub struct Resources {
                    #(#fields)*
                }
            });

            items.push(quote! {
                #[allow(unsafe_code)]
                impl Resources {
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
        if has_resources {
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
            tys.push(quote!(#tname::Resources));
            exprs.push(quote!(#tname::Resources::new()));
        }

        let path = &task.path;
        let _tname = Ident::from(format!("_{}", tname));
        let export_name = LitStr::new(tname.as_ref(), Span::call_site());
        root.push(quote! {
            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            #[export_name = #export_name]
            pub unsafe extern "C" fn #_tname() {
                let f: fn(#(#tys,)*) = #path;

                f(#(#exprs,)*)
            }
        });

        root.push(quote!{
            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            mod #tname {
                #[allow(unused_imports)]
                use core::marker::PhantomData;

                #[allow(dead_code)]
                #[deny(const_err)]
                pub const CHECK_PRIORITY: (u8, u8) = (
                    #priority - 1,
                    (1 << ::#device::NVIC_PRIO_BITS) - #priority,
                );

                #(#items)*
            }
        });

        // after miri landed (?) rustc won't analyze `const` items unless they are used so we force
        // evaluation with this path statement
        main.push(quote!(#tname::CHECK_PRIORITY;));
    }
}
