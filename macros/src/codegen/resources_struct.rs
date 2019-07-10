use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::{ast::App, Context};

use crate::{analyze::Analysis, codegen::util};

pub fn codegen(
    ctxt: Context,
    priority: u8,
    needs_lt: &mut bool,
    app: &App,
    analysis: &Analysis,
) -> (TokenStream2, TokenStream2) {
    let mut lt = None;

    let resources = match ctxt {
        Context::Init(core) => &app.inits[&core].args.resources,
        Context::Idle(core) => &app.idles[&core].args.resources,
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.resources,
    };

    let mut fields = vec![];
    let mut values = vec![];
    let mut has_cfgs = false;

    for (name, access) in resources {
        let (res, expr) = app.resource(name).expect("UNREACHABLE");

        let cfgs = &res.cfgs;
        has_cfgs |= !cfgs.is_empty();

        let mut_ = if access.is_exclusive() {
            Some(quote!(mut))
        } else {
            None
        };
        let ty = &res.ty;

        if ctxt.is_init() {
            if !analysis.ownerships.contains_key(name) {
                // owned by `init`
                fields.push(quote!(
                    #(#cfgs)*
                    pub #name: &'static #mut_ #ty
                ));

                values.push(quote!(
                    #(#cfgs)*
                    #name: &#mut_ #name
                ));
            } else {
                // owned by someone else
                lt = Some(quote!('a));

                fields.push(quote!(
                    #(#cfgs)*
                    pub #name: &'a mut #ty
                ));

                values.push(quote!(
                    #(#cfgs)*
                    #name: &mut #name
                ));
            }
        } else {
            let ownership = &analysis.ownerships[name];

            if ownership.needs_lock(priority) {
                if mut_.is_none() {
                    lt = Some(quote!('a));

                    fields.push(quote!(
                        #(#cfgs)*
                        pub #name: &'a #ty
                    ));
                } else {
                    // resource proxy
                    lt = Some(quote!('a));

                    fields.push(quote!(
                        #(#cfgs)*
                        pub #name: resources::#name<'a>
                    ));

                    values.push(quote!(
                        #(#cfgs)*
                        #name: resources::#name::new(priority)

                    ));

                    continue;
                }
            } else {
                let lt = if ctxt.runs_once() {
                    quote!('static)
                } else {
                    lt = Some(quote!('a));
                    quote!('a)
                };

                if ownership.is_owned() || mut_.is_none() {
                    fields.push(quote!(
                        #(#cfgs)*
                        pub #name: &#lt #mut_ #ty
                    ));
                } else {
                    fields.push(quote!(
                        #(#cfgs)*
                        pub #name: &#lt mut #ty
                    ));
                }
            }

            let is_late = expr.is_none();
            if is_late {
                let expr = if mut_.is_some() {
                    quote!(&mut *#name.as_mut_ptr())
                } else {
                    quote!(&*#name.as_ptr())
                };

                values.push(quote!(
                    #(#cfgs)*
                    #name: #expr
                ));
            } else {
                values.push(quote!(
                    #(#cfgs)*
                    #name: &#mut_ #name
                ));
            }
        }
    }

    if lt.is_some() {
        *needs_lt = true;

        // the struct could end up empty due to `cfg`s leading to an error due to `'a` being unused
        if has_cfgs {
            fields.push(quote!(
                #[doc(hidden)]
                pub __marker__: core::marker::PhantomData<&'a ()>
            ));

            values.push(quote!(__marker__: core::marker::PhantomData))
        }
    }

    let core = ctxt.core(app);
    let cores = app.args.cores;
    let cfg_core = util::cfg_core(core, cores);
    let doc = format!("Resources `{}` has access to", ctxt.ident(app));
    let ident = util::resources_ident(ctxt, app);
    let item = quote!(
        #cfg_core
        #[allow(non_snake_case)]
        #[doc = #doc]
        pub struct #ident<#lt> {
            #(#fields,)*
        }
    );

    let arg = if ctxt.is_init() {
        None
    } else {
        Some(quote!(priority: &#lt rtfm::export::Priority))
    };
    let constructor = quote!(
        #cfg_core
        impl<#lt> #ident<#lt> {
            #[inline(always)]
            unsafe fn new(#arg) -> Self {
                #ident {
                    #(#values,)*
                }
            }
        }
    );

    (item, constructor)
}
