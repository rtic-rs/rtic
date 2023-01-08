//! RTIC application analysis

use core::cmp;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use indexmap::{IndexMap, IndexSet};
use syn::{Ident, Type};

use crate::syntax::{
    ast::{App, LocalResources, TaskLocal},
    Set,
};

pub(crate) fn app(app: &App) -> Result<Analysis, syn::Error> {
    // Collect all tasks into a vector
    type TaskName = Ident;
    type Priority = u8;

    // The task list is a Tuple (Name, Shared Resources, Local Resources, Priority)
    let task_resources_list: Vec<(TaskName, Vec<&Ident>, &LocalResources, Priority)> =
        Some(&app.init)
            .iter()
            .map(|ht| (ht.name.clone(), Vec::new(), &ht.args.local_resources, 0))
            .chain(app.idle.iter().map(|ht| {
                (
                    ht.name.clone(),
                    ht.args
                        .shared_resources
                        .iter()
                        .map(|(v, _)| v)
                        .collect::<Vec<_>>(),
                    &ht.args.local_resources,
                    0,
                )
            }))
            .chain(app.software_tasks.iter().map(|(name, ht)| {
                (
                    name.clone(),
                    ht.args
                        .shared_resources
                        .iter()
                        .map(|(v, _)| v)
                        .collect::<Vec<_>>(),
                    &ht.args.local_resources,
                    ht.args.priority,
                )
            }))
            .chain(app.hardware_tasks.iter().map(|(name, ht)| {
                (
                    name.clone(),
                    ht.args
                        .shared_resources
                        .iter()
                        .map(|(v, _)| v)
                        .collect::<Vec<_>>(),
                    &ht.args.local_resources,
                    ht.args.priority,
                )
            }))
            .collect();

    let mut error = vec![];
    let mut lf_res_with_error = vec![];
    let mut lf_hash = HashMap::new();

    // Collect lock free resources
    let lock_free: Vec<&Ident> = app
        .shared_resources
        .iter()
        .filter(|(_, r)| r.properties.lock_free)
        .map(|(i, _)| i)
        .collect();

    // Check that lock_free resources are correct
    for lf_res in lock_free.iter() {
        for (task, tr, _, priority) in task_resources_list.iter() {
            for r in tr {
                // Get all uses of resources annotated lock_free
                if lf_res == r {
                    // Check so async tasks do not use lock free resources
                    if app.software_tasks.get(task).is_some() {
                        error.push(syn::Error::new(
                            r.span(),
                            format!(
                                "Lock free shared resource {:?} is used by an async tasks, which is forbidden",
                                r.to_string(),
                            ),
                        ));
                    }

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
            error.push(syn::Error::new(
                r.span(),
                format!(
                    "Lock free shared resource {:?} is used by tasks at different priorities",
                    r.to_string(),
                ),
            ));
        }
    }

    // Add error message for each use of the shared resource
    for resource in lf_res_with_error.clone() {
        error.push(syn::Error::new(
            resource.span(),
            format!(
                "Shared resource {:?} is declared lock free but used by tasks at different priorities",
                resource.to_string(),
            ),
        ));
    }

    // Collect local resources
    let local: Vec<&Ident> = app.local_resources.iter().map(|(i, _)| i).collect();

    let mut lr_with_error = vec![];
    let mut lr_hash = HashMap::new();

    // Check that local resources are not shared
    for lr in local {
        for (task, _, local_resources, _) in task_resources_list.iter() {
            for (name, res) in local_resources.iter() {
                // Get all uses of resources annotated lock_free
                if lr == name {
                    match res {
                        TaskLocal::External => {
                            // HashMap returns the previous existing object if old.key == new.key
                            if let Some(lr) = lr_hash.insert(name.to_string(), (task, name)) {
                                lr_with_error.push(lr.1);
                                lr_with_error.push(name);
                            }
                        }
                        // If a declared local has the same name as the `#[local]` struct, it's an
                        // direct error
                        TaskLocal::Declared(_) => {
                            lr_with_error.push(lr);
                            lr_with_error.push(name);
                        }
                    }
                }
            }
        }
    }

    // Add error message for each use of the local resource
    for resource in lr_with_error.clone() {
        error.push(syn::Error::new(
            resource.span(),
            format!(
                "Local resource {:?} is used by multiple tasks or collides with multiple definitions",
                resource.to_string(),
            ),
        ));
    }

    // Check 0-priority async software tasks and idle dependency
    for (name, task) in &app.software_tasks {
        if task.args.priority == 0 {
            // If there is a 0-priority task, there must be no idle
            if app.idle.is_some() {
                error.push(syn::Error::new(
                    name.span(),
                    format!(
                        "Async task {:?} has priority 0, but `#[idle]` is defined. 0-priority async tasks are only allowed if there is no `#[idle]`.",
                        name.to_string(),
                    )
                ));
            }
        }
    }

    // Collect errors if any and return/halt
    if !error.is_empty() {
        let mut err = error.get(0).unwrap().clone();
        error.iter().for_each(|e| err.combine(e.clone()));
        return Err(err);
    }

    // e. Location of resources
    let mut used_shared_resource = IndexSet::new();
    let mut ownerships = Ownerships::new();
    let mut sync_types = SyncTypes::new();
    for (prio, name, access) in app.shared_resource_accesses() {
        let res = app.shared_resources.get(name).expect("UNREACHABLE");

        // (e)
        // This shared resource is used
        used_shared_resource.insert(name.clone());

        // (c)
        if let Some(priority) = prio {
            if let Some(ownership) = ownerships.get_mut(name) {
                match *ownership {
                    Ownership::Owned { priority: ceiling }
                    | Ownership::CoOwned { priority: ceiling }
                    | Ownership::Contended { ceiling }
                        if priority != ceiling =>
                    {
                        *ownership = Ownership::Contended {
                            ceiling: cmp::max(ceiling, priority),
                        };

                        if access.is_shared() {
                            sync_types.insert(res.ty.clone());
                        }
                    }

                    Ownership::Owned { priority: ceil } if ceil == priority => {
                        *ownership = Ownership::CoOwned { priority };
                    }

                    _ => {}
                }
            } else {
                ownerships.insert(name.clone(), Ownership::Owned { priority });
            }
        }
    }

    // Create the list of used local resource Idents
    let mut used_local_resource = IndexSet::new();

    for (_, _, locals, _) in task_resources_list {
        for (local, _) in locals {
            used_local_resource.insert(local.clone());
        }
    }

    // Most shared resources need to be `Send`
    let mut send_types = SendTypes::new();
    let owned_by_idle = Ownership::Owned { priority: 0 };
    for (name, res) in app.shared_resources.iter() {
        // Handle not owned by idle
        if ownerships
            .get(name)
            .map(|ownership| *ownership != owned_by_idle)
            .unwrap_or(false)
        {
            send_types.insert(res.ty.clone());
        }
    }

    // Most local resources need to be `Send` as well
    for (name, res) in app.local_resources.iter() {
        if let Some(idle) = &app.idle {
            // Only Send if not in idle or not at idle prio
            if idle.args.local_resources.get(name).is_none()
                && !ownerships
                    .get(name)
                    .map(|ownership| *ownership != owned_by_idle)
                    .unwrap_or(false)
            {
                send_types.insert(res.ty.clone());
            }
        } else {
            send_types.insert(res.ty.clone());
        }
    }

    let mut channels = Channels::new();

    for (name, spawnee) in &app.software_tasks {
        let spawnee_prio = spawnee.args.priority;

        let channel = channels.entry(spawnee_prio).or_default();
        channel.tasks.insert(name.clone());
    }

    // No channel should ever be empty
    debug_assert!(channels.values().all(|channel| !channel.tasks.is_empty()));

    Ok(Analysis {
        channels,
        shared_resources: used_shared_resource,
        local_resources: used_local_resource,
        ownerships,
        send_types,
        sync_types,
    })
}

// /// Priority ceiling
// pub type Ceiling = Option<u8>;

/// Task priority
pub type Priority = u8;

/// Resource name
pub type Resource = Ident;

/// Task name
pub type Task = Ident;

/// The result of analyzing an RTIC application
pub struct Analysis {
    /// SPSC message channels
    pub channels: Channels,

    /// Shared resources
    ///
    /// If a resource is not listed here it means that's a "dead" (never
    /// accessed) resource and the backend should not generate code for it
    pub shared_resources: UsedSharedResource,

    /// Local resources
    ///
    /// If a resource is not listed here it means that's a "dead" (never
    /// accessed) resource and the backend should not generate code for it
    pub local_resources: UsedLocalResource,

    /// Resource ownership
    pub ownerships: Ownerships,

    /// These types must implement the `Send` trait
    pub send_types: SendTypes,

    /// These types must implement the `Sync` trait
    pub sync_types: SyncTypes,
}

/// All channels, keyed by dispatch priority
pub type Channels = BTreeMap<Priority, Channel>;

/// Location of all *used* shared resources
pub type UsedSharedResource = IndexSet<Resource>;

/// Location of all *used* local resources
pub type UsedLocalResource = IndexSet<Resource>;

/// Resource ownership
pub type Ownerships = IndexMap<Resource, Ownership>;

/// These types must implement the `Send` trait
pub type SendTypes = Set<Box<Type>>;

/// These types must implement the `Sync` trait
pub type SyncTypes = Set<Box<Type>>;

/// A channel used to send messages
#[derive(Debug, Default)]
pub struct Channel {
    /// The channel capacity
    pub capacity: u8,

    /// Tasks that can be spawned on this channel
    pub tasks: BTreeSet<Task>,
}

/// Resource ownership
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ownership {
    /// Owned by a single task
    Owned {
        /// Priority of the task that owns this resource
        priority: u8,
    },

    /// "Co-owned" by more than one task; all of them have the same priority
    CoOwned {
        /// Priority of the tasks that co-own this resource
        priority: u8,
    },

    /// Contended by more than one task; the tasks have different priorities
    Contended {
        /// Priority ceiling
        ceiling: u8,
    },
}

// impl Ownership {
//     /// Whether this resource needs to a lock at this priority level
//     pub fn needs_lock(&self, priority: u8) -> bool {
//         match self {
//             Ownership::Owned { .. } | Ownership::CoOwned { .. } => false,
//
//             Ownership::Contended { ceiling } => {
//                 debug_assert!(*ceiling >= priority);
//
//                 priority < *ceiling
//             }
//         }
//     }
//
//     /// Whether this resource is exclusively owned
//     pub fn is_owned(&self) -> bool {
//         matches!(self, Ownership::Owned { .. })
//     }
// }
