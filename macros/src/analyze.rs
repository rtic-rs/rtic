use std::cmp;
use std::collections::HashMap;

use syn::Ident;

use check::App;

pub type Ownerships = HashMap<Ident, Ownership>;

pub enum Ownership {
    /// Owned or co-owned by tasks that run at the same priority
    Owned { priority: u8 },
    /// Shared by tasks that run at different priorities.
    ///
    /// `ceiling` is the maximum value across all the task priorities
    Shared { ceiling: u8 },
}

impl Ownership {
    pub fn ceiling(&self) -> u8 {
        match *self {
            Ownership::Owned { priority } => priority,
            Ownership::Shared { ceiling } => ceiling,
        }
    }

    pub fn is_owned(&self) -> bool {
        match *self {
            Ownership::Owned { .. } => true,
            _ => false,
        }
    }
}

pub fn app(app: &App) -> Ownerships {
    let mut ownerships = HashMap::new();

    for resource in &app.idle.resources {
        ownerships.insert(resource.clone(), Ownership::Owned { priority: 0 });
    }

    for task in app.tasks.values() {
        for resource in task.resources.iter() {
            if let Some(ownership) = ownerships.get_mut(resource) {
                match *ownership {
                    Ownership::Owned { priority } => {
                        if priority == task.priority {
                            *ownership = Ownership::Owned { priority };
                        } else {
                            *ownership = Ownership::Shared {
                                ceiling: cmp::max(priority, task.priority),
                            };
                        }
                    }
                    Ownership::Shared { ceiling } => {
                        if task.priority > ceiling {
                            *ownership = Ownership::Shared {
                                ceiling: task.priority,
                            };
                        }
                    }
                }

                continue;
            }

            ownerships.insert(
                resource.clone(),
                Ownership::Owned {
                    priority: task.priority,
                },
            );
        }
    }

    ownerships
}
