use crate::syntax::{
    ast::{App, TaskLocal},
    Context,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::codegen::util;

/// Generates local resources structs
pub fn codegen(ctxt: Context, app: &App) -> (TokenStream2, TokenStream2) {
    let resources = match ctxt {
        Context::Init => &app.init.args.local_resources,
        Context::Idle => {
            &app.idle
                .as_ref()
                .expect("RTIC-ICE: unable to get idle name")
                .args
                .local_resources
        }
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.local_resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.local_resources,
    };

    let task_name = util::get_task_name(ctxt, app);

    let mut fields = vec![];
    let mut values = vec![];

    for (name, task_local) in resources {
        let (cfgs, ty, is_declared) = match task_local {
            TaskLocal::External => {
                let r = app.local_resources.get(name).expect("UNREACHABLE");
                (&r.cfgs, &r.ty, false)
            }
            TaskLocal::Declared(r) => (&r.cfgs, &r.ty, true),
        };

        let lt = if ctxt.runs_once() {
            quote!('static)
        } else {
            quote!('a)
        };

        let mangled_name = if matches!(task_local, TaskLocal::External) {
            util::static_local_resource_ident(name)
        } else {
            util::declared_static_local_resource_ident(name, &task_name)
        };

        fields.push(quote!(
            #(#cfgs)*
            pub #name: &#lt mut #ty
        ));

        let expr = if is_declared {
            // If the local resources is already initialized, we only need to access its value and
            // not go through an `MaybeUninit`
            quote!(&mut *#mangled_name.get_mut())
        } else {
            quote!(&mut *(&mut *#mangled_name.get_mut()).as_mut_ptr())
        };

        values.push(quote!(
            #(#cfgs)*
            #name: #expr
        ));
    }

    fields.push(quote!(
        #[doc(hidden)]
        pub __rtic_internal_marker: ::core::marker::PhantomData<&'a ()>
    ));

    values.push(quote!(__rtic_internal_marker: ::core::marker::PhantomData));

    let doc = format!("Local resources `{}` has access to", ctxt.ident(app));
    let ident = util::local_resources_ident(ctxt, app);
    let item = quote!(
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[doc = #doc]
        pub struct #ident<'a> {
            #(#fields,)*
        }
    );

    let constructor = quote!(
        impl<'a> #ident<'a> {
            #[inline(always)]
            pub unsafe fn new() -> Self {
                #ident {
                    #(#values,)*
                }
            }
        }
    );

    (item, constructor)
}
