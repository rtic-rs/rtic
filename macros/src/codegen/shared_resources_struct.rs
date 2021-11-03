use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{analyze::Ownership, ast::App, Context};

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generate shared resources structs
pub fn codegen(
    ctxt: Context,
    needs_lt: &mut bool,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (TokenStream2, TokenStream2) {
    let mut lt = None;

    let resources = match ctxt {
        Context::Init => unreachable!("Tried to generate shared resources struct for init"),
        Context::Idle => &app.idle.as_ref().unwrap().args.shared_resources,
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.shared_resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.shared_resources,
    };

    let mut fields = vec![];
    let mut values = vec![];
    let mut has_cfgs = false;

    // Lock-all api related
    let mut fields_mut = vec![];
    let mut values_mut = vec![];
    let mut max_ceiling = 0;
    let mut field_get_prio = None;

    for (name, access) in resources {
        let res = app.shared_resources.get(name).expect("UNREACHABLE");

        let cfgs = &res.cfgs;
        has_cfgs |= !cfgs.is_empty();

        // access hold if the resource is [x] (exclusive) or [&x] (shared)
        let mut_ = if access.is_exclusive() {
            Some(quote!(mut))
        } else {
            None
        };
        let ty = &res.ty;
        let mangled_name = util::static_shared_resource_ident(&name);

        if !res.properties.lock_free {
            lt = Some(quote!('a));
            if access.is_shared() {
                // [&x] (shared)
                fields.push(quote!(
                    #(#cfgs)*
                    pub #name: &'a #ty
                ));
            } else {
                // Resource proxy
                fields.push(quote!(
                    #(#cfgs)*
                    pub #name: shared_resources::#name<'a>
                ));

                field_get_prio = Some(quote!(
                    #name
                ));

                values.push(quote!(
                    #(#cfgs)*
                    #name: shared_resources::#name::new(priority)
                ));

                // Lock-all related
                fields_mut.push(quote!(
                    #(#cfgs)*
                    pub #name: &#lt mut #ty
                ));

                values_mut.push(quote!(
                    #(#cfgs)*
                    #name: &mut *(&mut *#mangled_name.get_mut()).as_mut_ptr()
                ));

                let ceiling = match analysis.ownerships.get(name) {
                    Some(Ownership::Owned { priority }) => *priority,
                    Some(Ownership::CoOwned { priority }) => *priority,
                    Some(Ownership::Contended { ceiling }) => *ceiling,
                    None => 0,
                };

                max_ceiling = std::cmp::max(ceiling, max_ceiling);

                // continue as the value has been filled,
                continue;
            }
        } else {
            let lt = if ctxt.runs_once() {
                quote!('static)
            } else {
                lt = Some(quote!('a));
                quote!('a)
            };

            fields.push(quote!(
                #(#cfgs)*
                pub #name: &#lt #mut_ #ty
            ));
        }

        let expr = if access.is_exclusive() {
            quote!(&mut *(&mut *#mangled_name.get_mut()).as_mut_ptr())
        } else {
            quote!(&*(&*#mangled_name.get()).as_ptr())
        };

        values.push(quote!(
            #(#cfgs)*
            #name: #expr
        ));
    }

    if lt.is_some() {
        *needs_lt = true;

        // The struct could end up empty due to `cfg`s leading to an error due to `'a` being unused
        if has_cfgs {
            fields.push(quote!(
                #[doc(hidden)]
                pub __marker__: core::marker::PhantomData<&'a ()>
            ));

            values.push(quote!(__marker__: core::marker::PhantomData))
        }
    }

    let doc = format!("Shared resources `{}` has access to", ctxt.ident(app));
    let ident = util::shared_resources_ident(ctxt, app);

    // Lock-all related
    let doc_mut = format!(
        "Shared resources `{}` has lock all access to",
        ctxt.ident(app)
    );
    let ident_mut = util::shared_resources_ident_mut(ctxt, app);

    let mut items = vec![];
    items.push(quote!(
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[doc = #doc]
        pub struct #ident<#lt> {
            #(#fields,)*
        }
    ));

    let arg = if ctxt.is_init() {
        None
    } else {
        Some(quote!(priority: &#lt rtic::export::Priority))
    };

    let (lock_all, new_struct, get_prio) = if let Some(name) = field_get_prio {
        items.push(quote!(
            // Used by the lock-all API
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[doc = #doc_mut]
            pub struct #ident_mut<#lt> {
                #(#fields_mut,)*
            }
        ));
        (
            util::impl_mutex_struct(
                extra,
                &vec![], // TODO: what cfg should go here?
                quote!(#ident),
                quote!(#ident_mut<#lt>),
                max_ceiling,
                quote!(self.priority()),
                quote!(|| { #ident_mut::new() }),
            ),
            quote!(
                // Used by the lock-all API
                impl<#lt> #ident_mut<#lt> {
                    #[inline(always)]
                    pub unsafe fn new() -> Self {
                        #ident_mut {
                            #(#values_mut,)*
                        }
                    }
                }
            ),
            quote!(
                // Used by the lock-all API
                #[inline(always)]
                pub unsafe fn priority(&self) -> &rtic::export::Priority {
                    self.#name.priority()
                }
            ),
        )
    } else {
        items.push(quote!(
            // Used by the lock-all API
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[doc = #doc_mut]
            pub struct #ident_mut {}
        ));
        (quote!(), quote!(), quote!())
    };

    let implementations = quote!(
        impl<#lt> #ident<#lt> {
            #[inline(always)]
            pub unsafe fn new(#arg) -> Self {
                #ident {
                    #(#values,)*
                }
            }

            #get_prio
        }

        #new_struct

        #lock_all
    );

    (quote!(#(#items)*), implementations)
}
