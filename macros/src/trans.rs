use quote::{Ident, Tokens};

use analyze::{Ownership, Ownerships};
use check::App;

fn krate() -> Ident {
    Ident::from("rtfm")
}

pub fn app(app: &App, ownerships: &Ownerships) -> Tokens {
    let mut root = vec![];
    let mut main = vec![];

    // ::trans::check(app, &mut main);
    ::trans::init(app, &mut main, &mut root);
    ::trans::idle(app, ownerships, &mut main, &mut root);
    ::trans::resources(app, ownerships, &mut root);
    ::trans::tasks(app, ownerships, &mut root);

    root.push(quote! {
        fn main() {
            #(#main)*
        }
    });

    quote!(#(#root)*)
}

// Check that the exceptions / interrupts are valid
// Sadly we can't do this test at expansion time. Instead we'll generate some
// code that won't compile if the interrupt name is invalid.
// fn check(app: &App, main: &mut Vec<Tokens>) {
// }

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

    if !app.idle.resources.is_empty() &&
        !app.idle
            .resources
            .iter()
            .all(|resource| ownerships[resource].is_owned())
    {
        tys.push(quote!(#krate::Threshold));
        exprs.push(quote!(unsafe { #krate::Threshold::new(0) }));
    }

    if !app.idle.locals.is_empty() {
        let mut lexprs = vec![];
        let mut lfields = vec![];

        for (name, resource) in &app.idle.locals {
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
            pub struct Locals {
                #(#lfields)*
            }
        });

        tys.push(quote!(&'static mut idle::Locals));
        exprs.push(quote!(unsafe { &mut LOCALS }));

        main.push(quote! {
            static mut LOCALS: idle::Locals = idle::Locals {
                #(#lexprs)*
            };
        });
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

                    rexprs.push(quote! {
                        #name: &mut *#super_::#name.get(),
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

fn init(app: &App, main: &mut Vec<Tokens>, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    let mut tys = vec![quote!(#device::Peripherals)];
    let mut exprs = vec![quote!(#device::Peripherals::all())];
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
        if let Some(enabled) = task.enabled {
            // Interrupt. These can be enabled / disabled through the NVIC
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
        } else {
            // Exception
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

fn resources(app: &App, ownerships: &Ownerships, root: &mut Vec<Tokens>) {
    let krate = krate();
    let device = &app.device;

    let mut items = vec![];
    let mut impls = vec![];
    for (name, ownership) in ownerships {
        let mut impl_items = vec![];

        match *ownership {
            Ownership::Owned { .. } => {
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
                    continue;
                }
            }
            Ownership::Shared { ceiling } => {
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
                            cs: &'cs #krate::CriticalSection,
                        ) -> &'cs #krate::Static<#ty> {
                            unsafe { #name.borrow(cs) }
                        }

                        pub fn borrow_mut<'cs>(
                            &'cs mut self,
                            cs: &'cs #krate::CriticalSection,
                        ) -> &'cs mut #krate::Static<#ty> {
                            unsafe { #name.borrow_mut(cs) }
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
                            cs: &'cs #krate::CriticalSection,
                        ) -> &'cs #device::#name {
                            unsafe { #name.borrow(cs) }
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

fn tasks(app: &App, ownerships: &Ownerships, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    for (name, task) in &app.tasks {
        let mut exprs = vec![];
        let mut fields = vec![];
        let mut items = vec![];

        let mut lifetime = None;
        let mut needs_reexport = false;
        for name in &task.resources {
            match ownerships[name] {
                Ownership::Shared { ceiling } if ceiling > task.priority => {
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
