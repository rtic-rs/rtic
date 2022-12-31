use core::ops;
use std::collections::{BTreeMap, BTreeSet};

use crate::syntax::{
    analyze::{self, Priority},
    ast::{App, Dispatcher},
};
use syn::Ident;

/// Extend the upstream `Analysis` struct with our field
pub struct Analysis {
    parent: analyze::Analysis,
    pub interrupts_normal: BTreeMap<Priority, (Ident, Dispatcher)>,
    pub interrupts_async: BTreeMap<Priority, (Ident, Dispatcher)>,
}

impl ops::Deref for Analysis {
    type Target = analyze::Analysis;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

// Assign an interrupt to each priority level
pub fn app(analysis: analyze::Analysis, app: &App) -> Analysis {
    let mut available_interrupt = app.args.dispatchers.clone();

    // the set of priorities (each priority only once)
    let priorities = app
        .software_tasks
        .values()
        .filter(|task| !task.is_async)
        .map(|task| task.args.priority)
        .collect::<BTreeSet<_>>();

    let priorities_async = app
        .software_tasks
        .values()
        .filter(|task| task.is_async)
        .map(|task| task.args.priority)
        .collect::<BTreeSet<_>>();

    // map from priorities to interrupts (holding name and attributes)

    let interrupts_normal: BTreeMap<Priority, _> = priorities
        .iter()
        .copied()
        .rev()
        .map(|p| (p, available_interrupt.pop().expect("UNREACHABLE")))
        .collect();

    let interrupts_async: BTreeMap<Priority, _> = priorities_async
        .iter()
        .copied()
        .rev()
        .map(|p| (p, available_interrupt.pop().expect("UNREACHABLE")))
        .collect();

    Analysis {
        parent: analysis,
        interrupts_normal,
        interrupts_async,
    }
}
