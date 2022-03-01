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
    pub interrupts: BTreeMap<Priority, (Ident, ExternInterrupt)>,
}

impl ops::Deref for Analysis {
    type Target = analyze::Analysis;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

// Assign an interrupt to each priority level
pub fn app(analysis: P<analyze::Analysis>, app: &App) -> P<Analysis> {
    // the set of priorities (each priority only once)
    let priorities = app
        .software_tasks
        .values()
        .map(|task| task.args.priority)
        .collect::<BTreeSet<_>>();

    // map from priorities to interrupts (holding name and attributes)
    let interrupts: BTreeMap<Priority, _> = priorities
        .iter()
        .copied()
        .rev()
        .zip(&app.args.extern_interrupts)
        .map(|(p, (id, ext))| (p, (id.clone(), ext.clone())))
        .collect();

    P::new(Analysis {
        parent: analysis,
        interrupts,
    })
}
