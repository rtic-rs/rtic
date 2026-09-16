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
    pub interrupts: BTreeMap<Priority, (Ident, Dispatcher)>,
    pub max_async_prio: Option<u8>,
}

impl ops::Deref for Analysis {
    type Target = analyze::Analysis;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

// Assign an interrupt to each priority level
pub fn app(analysis: analyze::Analysis, app: &App) -> Analysis {
    let mut available_dispatchers = app.args.dispatchers.clone();

    // the set of priorities (each priority only once)
    let priorities = app
        .software_tasks
        .values()
        .map(|task| task.args.priority)
        .collect::<BTreeSet<_>>();

    // map from priorities to interrupts (holding name and attributes)

    let nonzero_priorities = priorities
        .iter()
        // 0 prio tasks are run in main
        .filter(|prio| **prio > 0);
    assert!(
        available_dispatchers.len() >= nonzero_priorities.clone().count(),
        "The number of dispatchers must be equal to or greater than the number of distinct task priorities."
    );
    let interrupts: BTreeMap<Priority, _> = nonzero_priorities
        .copied()
        .rev()
        .map(|p| {
            (
                p,
                available_dispatchers
                    .pop()
                    // EXPECT: covered by above assertion
                    .expect("UNREACHABLE"),
            )
        })
        .collect();

    // Async HAL interrupts (e.g. monotonics) must be able to preempt every software task, and
    // should stay below the hardware tasks when the priority ranges allow it.
    let max_sw_prio = app
        .software_tasks
        .values()
        .map(|task| task.args.priority)
        .max();
    let max_async_prio = app
        .hardware_tasks
        .values()
        .map(|task| task.args.priority)
        .min()
        .map(|min_hw_prio| {
            let above_sw = max_sw_prio.map_or(0, |prio| prio.saturating_add(1));
            (min_hw_prio - 1).max(above_sw)
        });

    Analysis {
        parent: analysis,
        interrupts,
        max_async_prio,
    }
}
