use syn::parse;
use std::collections::HashMap;
use proc_macro2::Ident;
use rtfm_syntax::{
    analyze::Analysis,
    ast::App,
};
use syn::Error;

type Idents<'a> = Vec<&'a Ident>;

// Assign an `extern` interrupt to each priority level
pub fn app(app: &App, _analysis: &Analysis) -> parse::Result<()> {
    // collect task local resources
    let task_local: Idents = app
        .resources
        .iter()
        .filter(|(_, r)| r.properties.task_local)
        .map(|(i, _)| i)
        .chain(
            app.late_resources
                .iter()
                .filter(|(_, r)| r.properties.task_local)
                .map(|(i, _)| i),
        )
        .collect();

    let lock_free: Idents = app
        .resources
        .iter()
        .filter(|(_, r)| r.properties.lock_free)
        .map(|(i, _)| i)
        .chain(
            app.late_resources
                .iter()
                .filter(|(_, r)| r.properties.lock_free)
                .map(|(i, _)| i),
        )
        .collect();

    // collect all tasks into a vector
    type Task = String;
    type Priority = u8;

    let all_tasks: Vec<(Task, Idents, Priority)> = app
        .idles
        .iter()
        .map(|(core, ht)| {
            (
                format!("Idle (core {})", core),
                ht.args.resources.iter().map(|(v, _)| v).collect::<Vec<_>>(),
                0

            )
        })
        .chain(app.software_tasks.iter().map(|(name, ht)| {
            (
                name.to_string(),
                ht.args.resources.iter().map(|(v, _)| v).collect::<Vec<_>>(),
                ht.args.priority
            )
        }))
        .chain(app.hardware_tasks.iter().map(|(name, ht)| {
            (
                name.to_string(),
                ht.args.resources.iter().map(|(v, _)| v).collect::<Vec<_>>(),
                ht.args.priority
            )
        }))
        .collect();

    // check that task_local resources is only used once
    let mut error = vec![];
    for task_local_id in task_local.iter() {
        let mut used = vec![];
        for (task, tr, priority) in all_tasks.iter() {
            for r in tr {
                if task_local_id == r {
                    used.push((task, r, priority));
                }
            }
        }
        if used.len() > 1 {
            error.push(Error::new(
                task_local_id.span(),
                format!(
                    "task local resource {:?} is used by multiple tasks",
                    task_local_id.to_string()
                ),
            ));

            used.iter().for_each(|(task, resource, priority)| {
                error.push(Error::new(
                    resource.span(),
                    format!(
                        "task local resource {:?} is used by task {:?} with priority {:?}",
                        resource.to_string(),
                        task,
                        priority
                    ),
                ))
            });
        }
    }

    let mut lf_res_with_error = vec![];
    let mut lf_hash = HashMap::new();

    for lf_res in lock_free.iter() {
        for (task, tr, priority) in all_tasks.iter() {
            for r in tr {
                // Get all uses of resources annotated lock_free
                if lf_res == r {
                    // HashMap returns the previous existing object if old.key == new.key
                    if let Some(lf_res) = lf_hash.insert(r.to_string(), (task, r, priority)) {
                        // Check if priority differ, if it does, append to
                        // list of resources which will be annotated with errors
                        if priority != lf_res.2 {
                            lf_res_with_error.push(lf_res.1);
                            lf_res_with_error.push(r);
                        }
                        // If the resource already violates lock free properties
                        if lf_res_with_error.contains(&r) {
                            lf_res_with_error.push(lf_res.1);
                            lf_res_with_error.push(r);
                        }
                    }
                }
            }
        }
    }

    // Add error message in the resource struct
    for r in lock_free {
        if lf_res_with_error.contains(&&r) {
            error.push(Error::new(
                r.span(),
                format!(
                    "Lock free resource {:?} is used by tasks at different priorities",
                    r.to_string(),
                ),
            ));
        }
    }

    // Add error message for each use of the resource
    for resource in lf_res_with_error.clone() {
        error.push(Error::new(
            resource.span(),
            format!(
                "Resource {:?} is declared lock free but used by tasks at different priorities",
                resource.to_string(),
            ),
        ));
    }
    
    // collect errors
    if error.is_empty() {
        Ok(())
    } else {
        let mut err = error.iter().next().unwrap().clone();
        error.iter().for_each(|e| err.combine(e.clone()));
        Err(err)
    }
}