use core::ops;
use std::collections::{BTreeMap, BTreeSet};

use rtic_syntax::{
    analyze::{self, Priority},
    ast::App,
    P,
};
use syn::Ident;

/// Extend the upstream `Analysis` struct with our field
pub struct Analysis {
    parent: P<analyze::Analysis>,
    pub interrupts: BTreeMap<Priority, Ident>,
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
        let priorities = app
            .software_tasks
            .values()
            .filter_map(|task| {
                    Some(task.args.priority)
            })
            .chain(analysis.timer_queues.first().map(|tq| tq.priority))
            .collect::<BTreeSet<_>>();

        if !priorities.is_empty() {
            interrupts =
                priorities
                    .iter()
                    .cloned()
                    .rev()
                    .zip(app.extern_interrupts.keys().cloned())
                    .collect();
        }

    P::new(Analysis {
        parent: analysis,
        interrupts,
    })
}
