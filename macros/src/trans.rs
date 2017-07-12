use quote::Tokens;
use syn::Ident;

use syntax::{App, Kind};
use util::{Ceiling, Ceilings};

fn krate() -> Ident {
    Ident::new("rtfm")
}

pub fn app(app: &App, ceilings: &Ceilings) -> Tokens {
    let mut main = vec![];
    let mut root = vec![];

    super::trans::init(app, &mut main, &mut root);
    super::trans::idle(app, ceilings, &mut main, &mut root);
    super::trans::resources(app, ceilings, &mut root);
    super::trans::tasks(app, ceilings, &mut root);

    root.push(quote! {
        fn main() {
            #(#main)*
        }
    });

    quote!(#(#root)*)
}

fn init(app: &App, main: &mut Vec<Tokens>, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    let mut tys = vec![quote!(init::Peripherals)];
    let mut exprs = vec![quote!(init::Peripherals::all())];
    let mut mod_items = vec![];

    if !app.resources.is_empty() {
        let mut fields = vec![];
        let mut lifetime = None;
        let mut rexprs = vec![];

        for (name, resource) in &app.resources {
            lifetime = Some(quote!('a));

            let ty = &resource.ty;

            fields.push(quote! {
                pub #name: &'a mut #krate::Static<#ty>,
            });

            rexprs.push(quote! {
                #name: ::#krate::Static::ref_mut(&mut *super::#name.get()),
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

    root.push(quote! {
        mod init {
            pub use ::#device::Peripherals;

            #(#mod_items)*
        }
    });

    let mut exceptions = vec![];
    let mut interrupts = vec![];
    for (name, task) in &app.tasks {
        match task.kind {
            Kind::Exception => {
                if exceptions.is_empty() {
                    exceptions.push(quote! {
                        let scb = #device::SCB.borrow(_cs);
                    });
                }

                let priority = task.priority;
                exceptions.push(quote! {
                    let prio_bits = #device::NVIC_PRIO_BITS;
                    let hw = ((1 << prio_bits) - #priority) << (8 - prio_bits);
                    scb.shpr[#krate::Exception::#name.nr() - 4].write(hw);
                });
            }
            Kind::Interrupt { enabled } => {
                if interrupts.is_empty() {
                    interrupts.push(quote! {
                        let nvic = #device::NVIC.borrow(_cs);
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
        let init: fn(#(#tys,)*) = #init;

        #krate::atomic(|_cs| unsafe {
            init(#(#exprs,)*);

            #(#exceptions)*
            #(#interrupts)*
        });
    });
}

fn idle(
    app: &App,
    ceilings: &Ceilings,
    main: &mut Vec<Tokens>,
    root: &mut Vec<Tokens>,
) {
    let krate = krate();

    let mut mod_items = vec![];
    let mut tys = vec![];
    let mut exprs = vec![];

    if !app.idle.resources.is_empty() &&
        !app.idle
            .resources
            .iter()
            .all(|resource| ceilings[resource].is_owned())
    {
        tys.push(quote!(#krate::Threshold));
        exprs.push(quote!(unsafe { #krate::Threshold::new(0) }));
    }

    if !app.idle.local.is_empty() {
        let mut lexprs = vec![];
        let mut lfields = vec![];

        for (name, resource) in &app.idle.local {
            let expr = &resource.expr;
            let ty = &resource.ty;

            lfields.push(quote! {
                pub #name: #ty,
            });

            lexprs.push(quote! {
                #name: #expr,
            });
        }

        mod_items.push(quote! {
            pub struct Local {
                #(#lfields)*
            }
        });

        tys.push(quote!(&'static mut idle::Local));
        exprs.push(quote!(unsafe { &mut LOCAL }));

        main.push(quote! {
            static mut LOCAL: idle::Local = idle::Local {
                #(#lexprs)*
            };
        });
    }

    if !app.idle.resources.is_empty() {
        let device = &app.device;
        let mut lifetime = None;

        let mut needs_reexport = false;
        for name in &app.idle.resources {
            if ceilings[name].is_owned() {
                if app.resources.get(name).is_some() {
                    needs_reexport = true;
                    break
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
            if ceilings[name].is_owned() {
                lifetime = Some(quote!('a));
                if let Some(resource) = app.resources.get(name) {
                    let ty = &resource.ty;

                    rfields.push(quote! {
                        pub #name: &'a mut ::#krate::Static<#ty>,
                    });

                    rexprs.push(quote! {
                        #name: ::#krate::Static::ref_mut(
                            &mut *#super_::#name.get(),
                        ),
                    });
                } else {
                    rfields.push(quote! {
                        pub #name: &'a mut ::#device::#name,
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
                pub struct _idleResources<#lifetime> {
                    #(#rfields)*
                }
            });

            mod_items.push(quote! {
                pub use ::_idleResources as Resources;
            });
        } else {
            mod_items.push(quote! {
                #[allow(non_snake_case)]
                pub struct Resources<#lifetime> {
                    #(#rfields)*
                }
            });
        }

        mod_items.push(quote! {
            impl<#lifetime> Resources<#lifetime> {
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

    root.push(quote! {
        mod idle {
            #(#mod_items)*
        }
    });

    let idle = &app.idle.path;
    main.push(quote! {
        // type check
        let idle: fn(#(#tys),*) -> ! = #idle;

        idle(#(#exprs),*);
    });
}

fn tasks(app: &App, ceilings: &Ceilings, root: &mut Vec<Tokens>) {
    let krate = krate();

    for (name, task) in &app.tasks {
        let mut exprs = vec![];
        let mut fields = vec![];
        let mut items = vec![];

        let device = &app.device;
        let mut lifetime = None;
        let mut needs_reexport = false;
        for name in &task.resources {
            match ceilings[name] {
                Ceiling::Shared(ceiling) if ceiling > task.priority => {
                    fields.push(quote! {
                        pub #name: super::_resource::#name,
                    });

                    exprs.push(quote! {
                        #name: {
                            super::_resource::#name::new()
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

                        exprs.push(quote! {
                            #name: ::#krate::Static::ref_mut(
                                &mut *super::#name.get(),
                            ),
                        });
                    } else {
                        fields.push(quote! {
                            pub #name: &'a mut ::#device::#name,
                        });

                        exprs.push(quote! {
                            #name: &mut *::#device::#name.get(),
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
            impl<#lifetime> Resources<#lifetime> {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#exprs)*
                    }
                }
            }
        });

        let priority = task.priority;
        root.push(quote!{
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            mod #name {
                #[deny(dead_code)]
                pub const #name: u8 = #priority;
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

fn resources(app: &App, ceilings: &Ceilings, root: &mut Vec<Tokens>) {
    let krate = krate();
    let device = &app.device;

    let mut items = vec![];
    let mut impls = vec![];
    for (name, ceiling) in ceilings {
        let mut impl_items = vec![];

        match *ceiling {
            Ceiling::Owned(_) => {
                if let Some(resource) = app.resources.get(name) {
                    // For owned resources we don't need claim() or borrow(),
                    // just get()
                    let expr = &resource.expr;
                    let ty = &resource.ty;

                    root.push(quote! {
                        static #name: #krate::Resource<#ty> =
                            #krate::Resource::new(#expr);
                    });
                } else {
                    // Peripheral
                    continue
                }
            },
            Ceiling::Shared(ceiling) => {
                if let Some(resource) = app.resources.get(name) {
                    let expr = &resource.expr;
                    let ty = &resource.ty;

                    root.push(quote! {
                        static #name: #krate::Resource<#ty> =
                            #krate::Resource::new(#expr);
                    });

                    impl_items.push(quote! {
                        pub fn borrow<'cs>(
                            &'cs self,
                            _cs: &'cs #krate::CriticalSection,
                        ) -> &'cs #krate::Static<#ty> {
                            unsafe {
                                #krate::Static::ref_(&*#name.get())
                            }
                        }

                        pub fn borrow_mut<'cs>(
                            &'cs mut self,
                            _cs: &'cs #krate::CriticalSection,
                        ) -> &'cs mut #krate::Static<#ty> {
                            unsafe {
                                #krate::Static::ref_mut(&mut *#name.get())
                            }
                        }

                        pub fn claim<R, F>(
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
                                #name.claim(
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }

                        pub fn claim_mut<R, F>(
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
                                #name.claim_mut(
                                    #ceiling,
                                    #device::NVIC_PRIO_BITS,
                                    t,
                                    f,
                                )
                            }
                        }
                    });
                } else {
                    root.push(quote! {
                        static #name: #krate::Peripheral<#device::#name> =
                            #krate::Peripheral::new(#device::#name);
                    });

                    impl_items.push(quote! {
                        pub fn borrow<'cs>(
                            &'cs self,
                            _cs: &'cs #krate::CriticalSection,
                        ) -> &'cs #device::#name {
                            unsafe {
                                &*#name.get()
                            }
                        }

                        pub fn claim<R, F>(
                            &self,
                            t: &mut #krate::Threshold,
                            f: F,
                        ) -> R
                        where
                            F: FnOnce(
                                &#device::#name,
                                &mut #krate::Threshold) -> R
                        {
                            unsafe {
                                #name.claim(
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
                    #[allow(dead_code)]
                    impl _resource::#name {
                        #(#impl_items)*
                    }
                });

                items.push(quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #name { _0: () }

                    impl #name {
                        pub unsafe fn new() -> Self {
                            #name { _0: () }
                        }
                    }
                });
            }
        }
    }

    root.push(quote! {
        mod _resource {
            #(#items)*
        }

        #(#impls)*
    });
}
