use std::cmp;
use std::collections::HashMap;

use syn::Ident;

use syntax::App;

pub type Ceilings = HashMap<Ident, Ceiling>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ceiling {
    // Owned by one or more tasks that have the same priority
    Owned(u8),
    // Shared by tasks with different priorities
    Shared(u8),
}

impl Ceiling {
    pub fn is_owned(&self) -> bool {
        match *self {
            Ceiling::Owned(_) => true,
            _ => false,
        }
    }
}

pub fn compute_ceilings(app: &App) -> Ceilings {
    let mut ceilings = HashMap::new();

    for resource in &app.idle.resources {
        ceilings.insert(resource.clone(), Ceiling::Owned(0));
    }

    for task in app.tasks.values() {
        for resource in &task.resources {
            if let Some(ceiling) = ceilings.get_mut(resource) {
                match *ceiling {
                    Ceiling::Owned(current) => {
                        if current == task.priority {
                            *ceiling = Ceiling::Owned(current);
                        } else {
                            *ceiling = Ceiling::Shared(
                                cmp::max(current, task.priority),
                            );
                        }
                    }
                    Ceiling::Shared(old) => {
                        if task.priority > old {
                            *ceiling = Ceiling::Shared(task.priority);
                        }
                    }
                }

                continue;
            }

            ceilings.insert(resource.clone(), Ceiling::Owned(task.priority));
        }
    }

    ceilings
}
