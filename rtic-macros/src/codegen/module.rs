use crate::syntax::{ast::App, Context};
use crate::{analyze::Analysis, codegen::bindings::interrupt_mod, codegen::util};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[allow(clippy::too_many_lines)]
pub fn codegen(ctxt: Context, app: &App, analysis: &Analysis) -> TokenStream2 {
    let mut items = vec![];
    let mut module_items = vec![];
    let mut fields = vec![];
    let mut values = vec![];
    // Used to copy task cfgs to the whole module
    let mut task_cfgs = vec![];

    let name = ctxt.ident(app);

    match ctxt {
        Context::Init => {
            fields.push(quote!(
                /// The space used to allocate async executors in bytes.
                pub executors_size: usize
            ));

            if app.args.core {
                fields.push(quote!(
                    /// Core peripherals
                    pub core: rtic::export::Peripherals
                ));

                values.push(quote!(core: core));
            }

            if app.args.peripherals {
                let device = &app.args.device;

                fields.push(quote!(
                    /// Device peripherals (PAC)
                    pub device: #device::Peripherals
                ));

                values.push(quote!(device: #device::Peripherals::steal()));
            }

            fields.push(quote!(
                /// Critical section token for init
                pub cs: rtic::export::CriticalSection<'a>
            ));

            values.push(quote!(cs: rtic::export::CriticalSection::new()));
            values.push(quote!(executors_size));
        }

        Context::Idle | Context::HardwareTask(_) | Context::SoftwareTask(_) => {}
    }

    if ctxt.has_local_resources(app) {
        let ident = util::local_resources_ident(ctxt, app);

        module_items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as LocalResources;
        ));

        fields.push(quote!(
            /// Local Resources this task has access to
            pub local: #name::LocalResources<'a>
        ));

        values.push(quote!(local: #name::LocalResources::new()));
    }

    if ctxt.has_shared_resources(app) {
        let ident = util::shared_resources_ident(ctxt, app);

        module_items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as SharedResources;
        ));

        fields.push(quote!(
            /// Shared Resources this task has access to
            pub shared: #name::SharedResources<'a>
        ));

        values.push(quote!(shared: #name::SharedResources::new()));
    }

    let doc = match ctxt {
        Context::Idle => "Idle loop",
        Context::Init => "Initialization function",
        Context::HardwareTask(_) => "Hardware task",
        Context::SoftwareTask(_) => "Software task",
    };

    let v = Vec::new();
    let cfgs = match ctxt {
        Context::HardwareTask(t) => &app.hardware_tasks[t].cfgs,
        Context::SoftwareTask(t) => &app.software_tasks[t].cfgs,
        _ => &v,
    };

    let core = if ctxt.is_init() {
        if app.args.core {
            Some(quote!(core: rtic::export::Peripherals, executors_size: usize))
        } else {
            Some(quote!(executors_size: usize))
        }
    } else {
        None
    };

    let internal_context_name = util::internal_task_ident(name, "Context");
    let exec_name = util::internal_task_ident(name, "EXEC");

    if let Context::SoftwareTask(t) = ctxt {
        let spawnee = &app.software_tasks[name];
        let priority = spawnee.args.priority;
        let cfgs = &spawnee.cfgs;
        // Store a copy of the task cfgs
        task_cfgs.clone_from(cfgs);

        let pend_interrupt = if priority > 0 {
            let int_mod = interrupt_mod(app);
            let interrupt = &analysis.interrupts.get(&priority).expect("UREACHABLE").0;
            quote!(rtic::export::pend(#int_mod::#interrupt);)
        } else {
            quote!()
        };

        let internal_spawn_ident = util::internal_task_ident(name, "spawn");
        let internal_waker_ident = util::internal_task_ident(name, "waker");
        let from_ptr_n_args = util::from_ptr_n_args_ident(spawnee.inputs.len());
        let (input_args, input_tupled, input_untupled, input_ty) =
            util::regroup_inputs(&spawnee.inputs);

        let local_task = app.software_tasks[t].args.local_task;
        let unsafety = if local_task {
            // local tasks are only safe to call from the same executor
            quote! { unsafe }
        } else {
            quote! {}
        };

        // Spawn caller
        items.push(quote!(
            #(#cfgs)*
            /// Spawns the task directly
            #[allow(non_snake_case)]
            #[doc(hidden)]
            pub #unsafety fn #internal_spawn_ident(#(#input_args,)*) -> ::core::result::Result<(), #input_ty> {
                // SAFETY: If `try_allocate` succeeds one must call `spawn`, which we do.
                unsafe {
                    let exec = rtic::export::executor::AsyncTaskExecutor::#from_ptr_n_args(#name, &#exec_name);
                    if exec.try_allocate() {
                        exec.spawn(#name(unsafe { #name::Context::new() } #(,#input_untupled)*));
                        #pend_interrupt

                        Ok(())
                    } else {
                        Err(#input_tupled)
                    }
                }
            }
        ));

        // Waker
        items.push(quote!(
            #(#cfgs)*
            /// Gives waker to the task
            #[allow(non_snake_case)]
            #[doc(hidden)]
            pub fn #internal_waker_ident() -> ::core::task::Waker {
                // SAFETY: #exec_name is a valid pointer to an executor.
                unsafe {
                    let exec = rtic::export::executor::AsyncTaskExecutor::#from_ptr_n_args(#name, &#exec_name);
                    exec.waker(|| {
                        let exec = rtic::export::executor::AsyncTaskExecutor::#from_ptr_n_args(#name, &#exec_name);
                        exec.set_pending();
                        #pend_interrupt
                    })
                }
            }
        ));

        if !local_task {
            module_items.push(quote!(
                #(#cfgs)*
                #[doc(inline)]
                pub use super::#internal_spawn_ident as spawn;
            ));
        }

        let local_tasks_on_same_executor: Vec<_> = app
            .software_tasks
            .iter()
            .filter(|(_, t)| t.args.local_task && t.args.priority == priority)
            .collect();

        if !local_tasks_on_same_executor.is_empty() {
            let local_spawner = util::internal_task_ident(t, "LocalSpawner");
            fields.push(quote! {
                /// Used to spawn tasks on the same executor
                ///
                /// This is useful for tasks that take args which are !Send/!Sync.
                ///
                /// NOTE: This only works with tasks marked `local_task = true`
                /// and which have the same priority and thus will run on the
                /// same executor.
                pub local_spawner: #local_spawner
            });
            let tasks = local_tasks_on_same_executor
                .iter()
                .map(|(ident, task)| {
                    // Copied mostly from software_tasks.rs
                    let internal_spawn_ident = util::internal_task_ident(ident, "spawn");
                    let attrs = &task.attrs;
                    let cfgs = &task.cfgs;
                    let inputs = &task.inputs;
                    let generics = if task.is_bottom {
                        quote!()
                    } else {
                        quote!(<'a>)
                    };
                    let input_vals = inputs.iter().map(|i| &i.pat).collect::<Vec<_>>();
                    let (_input_args, _input_tupled, _input_untupled, input_ty) = util::regroup_inputs(&task.inputs);
                    quote! {
                        #(#attrs)*
                        #(#cfgs)*
                        #[allow(non_snake_case)]
                        pub(super) fn #ident #generics(&self #(,#inputs)*) -> ::core::result::Result<(), #input_ty> {
                            // SAFETY: This is safe to call since this can only be called
                            // from the same executor
                            unsafe { #internal_spawn_ident(#(#input_vals,)*) }
                        }
                    }
                })
                .collect::<Vec<_>>();
            values.push(quote!(local_spawner: #local_spawner { _p: core::marker::PhantomData }));
            items.push(quote! {
                struct #local_spawner {
                    _p: core::marker::PhantomData<*mut ()>,
                }

                impl #local_spawner {
                    #(#tasks)*
                }
            });
        }

        module_items.push(quote!(
            #(#cfgs)*
            #[doc(inline)]
            pub use super::#internal_waker_ident as waker;
        ));
    }

    items.push(quote!(
        #(#cfgs)*
        /// Execution context
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        pub struct #internal_context_name<'a> {
            #[doc(hidden)]
            __rtic_internal_p: ::core::marker::PhantomData<&'a ()>,
            #(#fields,)*
        }

        #(#cfgs)*
        impl<'a> #internal_context_name<'a> {
            #[inline(always)]
            #[allow(missing_docs)]
            pub unsafe fn new(#core) -> Self {
                #internal_context_name {
                    __rtic_internal_p: ::core::marker::PhantomData,
                    #(#values,)*
                }
            }
        }
    ));

    module_items.push(quote!(
        #(#cfgs)*
        #[doc(inline)]
        pub use super::#internal_context_name as Context;
    ));

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
