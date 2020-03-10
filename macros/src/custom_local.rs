use syn::parse;
//use syn::Ident;
//use proc_macro2::{Ident, Span};
use proc_macro2::Ident;
use rtfm_syntax::{
    analyze::{Analysis, Ownership},
    ast::App,
};
use syn::Error;
//     ast::{App, CustomArg},

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

    // filter out contended resources
    let contended: Vec<(&Ident, &Ownership)> = _analysis
        .ownerships
        .iter()
        .filter(|(_id, own)| match own {
            Ownership::Contended { .. } => true,
            _ => false,
        })
        .collect();

    // filter out lock_free contended resources
    let lock_free_violation: Vec<&(&Ident, &Ownership)> = contended
        .iter()
        .filter(|(cont_id, _)| lock_free.iter().any(|lf_id| cont_id == lf_id))
        .collect();

    // report contention error
    lock_free_violation.iter().for_each(|(lf_err_id, lf_own)| {
        error.push(Error::new(
            lf_err_id.span(),
            format!(
                "lock_free resource {:?} is contended by task {}",
                lf_err_id.to_string(),
                if let Ownership::Contended{ceiling} = lf_own {
                    // Match the task running at the same priority
                    let ct_task: Vec<&(Task, Idents, Priority)> = all_tasks
                    .iter()
                    .filter(|(_name, _ident, ct_prio)| {
                        ct_prio == ceiling
                    }
                    )
                    .collect();
                    format!(
                        "{:#?} with priority {:?}",
                        ct_task[0].0,
                        ct_task[0].2
                    )
                } else {
                    "at higher priority".to_string()
                }
            ),
        ));
    });

    // collect errors
    if error.is_empty() {
        Ok(())
    } else {
        let mut err = error.iter().next().unwrap().clone();
        error.iter().for_each(|e| err.combine(e.clone()));
        Err(err)
    }

    //     for tl in task_local {
    //         println!("tl {:?}", tl);
    //  //       let mut first_use = None;
    //         for i in _analysis.ownerships.iter() {
    //             println!("\nown: {:?}", i);
    //     }

    //     // println!("analysis {:?}", _analysis.locations);

    // for i in _analysis.locations.iter() {
    //     println!("\nloc: {:?}", i);
    // }

    // ErrorMessage {
    //     // Span is implemented as an index into a thread-local interner to keep the
    //     // size small. It is not safe to access from a different thread. We want
    //     // errors to be Send and Sync to play nicely with the Failure crate, so pin
    //     // the span we're given to its original thread and assume it is
    //     // Span::call_site if accessed from any other thread.
    //     start_span: ThreadBound<Span>,
    //     end_span: ThreadBound<Span>,
    //     message: String,
    // }
    //        (app.name, "here");
    // let span = app.name.span();
    // let start = span.start();
    //    Err(vec![ErrorMessage { start_span: app.name.}]);

    // let mut task_locals = Vec::new();
    // println!("-- app:late_resources");

    // println!(
    //     "task_locals {:?}",
    //     app.late_resources.filter(|r| r.task_local)
    // );

    // println!("-- resources");
    // for i in app.resources.iter() {
    //     println!("res: {:?}", i);
    // }
}
