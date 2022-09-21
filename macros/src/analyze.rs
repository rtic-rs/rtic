use core::ops;
use std::collections::{BTreeMap, BTreeSet};

use rtic_syntax::{
    analyze::{self, Priority},
    ast::{App, ExternInterrupt},
    P,
};
use syn::Ident;

/// Extend the upstream `Analysis` struct with our field
pub struct Analysis {
    parent: P<analyze::Analysis>,
    pub interrupts_normal: BTreeMap<Priority, (Ident, ExternInterrupt)>,
    pub interrupts_async: BTreeMap<Priority, (Ident, ExternInterrupt)>,
}

impl ops::Deref for Analysis {
    type Target = analyze::Analysis;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

// Assign an interrupt to each priority level
pub fn app(analysis: P<analyze::Analysis>, app: &App) -> P<Analysis> {
    let mut available_interrupt = app.args.extern_interrupts.clone();

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

    P::new(Analysis {
        parent: analysis,
        interrupts_normal,
        interrupts_async,
    })
}
