use core::ops;
use std::collections::{BTreeMap, BTreeSet};

use rtfm_syntax::{
    analyze::{self, Priority},
    ast::App,
    Core, P,
};
use syn::Ident;

/// Extend the upstream `Analysis` struct with our field
pub struct Analysis {
    parent: P<analyze::Analysis>,
    pub interrupts: BTreeMap<Core, BTreeMap<Priority, Ident>>,
}

impl ops::Deref for Analysis {
    type Target = analyze::Analysis;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

// Assign an `extern` interrupt to each priority level
pub fn app(analysis: P<analyze::Analysis>, app: &App) -> P<Analysis> {
    let mut interrupts = BTreeMap::new();
    for core in 0..app.args.cores {
        let priorities = app
            .software_tasks
            .values()
            .filter_map(|task| {
                if task.args.core == core {
                    Some(task.args.priority)
                } else {
                    None
                }
            })
            .chain(analysis.timer_queues.get(&core).map(|tq| tq.priority))
            .collect::<BTreeSet<_>>();

        if !priorities.is_empty() {
            interrupts.insert(
                core,
                priorities
                    .iter()
                    .cloned()
                    .rev()
                    .zip(app.extern_interrupts[&core].keys().cloned())
                    .collect(),
            );
        }
    }

    P::new(Analysis {
        parent: analysis,
        interrupts,
    })
}
