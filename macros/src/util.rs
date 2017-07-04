use std::collections::HashMap;

use syn::Ident;

use syntax::App;

pub type Ceilings = HashMap<Ident, Ceiling>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ceiling {
    Owned,
    Shared(u8),
}

impl Ceiling {
    pub fn is_owned(&self) -> bool {
        *self == Ceiling::Owned
    }
}

pub fn compute_ceilings(app: &App) -> Ceilings {
    let mut ceilings = HashMap::new();

    for resource in &app.idle.resources {
        ceilings.insert(resource.clone(), Ceiling::Owned);
    }

    for task in app.tasks.values() {
        for resource in &task.resources {
            if let Some(ceiling) = ceilings.get_mut(resource) {
                match *ceiling {
                    Ceiling::Owned => *ceiling = Ceiling::Shared(task.priority),
                    Ceiling::Shared(old) => {
                        if task.priority > old {
                            *ceiling = Ceiling::Shared(task.priority);
                        }
                    }
                }

                continue;
            }

            ceilings.insert(resource.clone(), Ceiling::Owned);
        }
    }

    ceilings
}
