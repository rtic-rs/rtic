use std::collections::HashMap;

use syn::{Ident, Path};
use syntax::check::{self, Idents, Idle, Init, Statics};
use syntax::{self, Result};

pub struct App {
    pub device: Path,
    pub idle: Idle,
    pub init: Init,
    pub resources: Statics,
    pub tasks: Tasks,
}

pub type Tasks = HashMap<Ident, Task>;

#[allow(non_camel_case_types)]
pub enum Exception {
    PendSV,
    SVCall,
    SysTick,
}

impl Exception {
    pub fn from(s: &str) -> Option<Self> {
        Some(match s {
            "PendSV" => Exception::PendSV,
            "SVCall" => Exception::SVCall,
            "SysTick" => Exception::SysTick,
            _ => return None,
        })
    }

    pub fn nr(&self) -> usize {
        match *self {
            Exception::PendSV => 14,
            Exception::SVCall => 11,
            Exception::SysTick => 15,
        }
    }
}

pub enum Kind {
    Exception(Exception),
    Interrupt { enabled: bool },
}

pub struct Task {
    pub kind: Kind,
    pub path: Path,
    pub priority: u8,
    pub resources: Idents,
}

pub fn app(app: check::App) -> Result<App> {
    let app = App {
        device: app.device,
        idle: app.idle,
        init: app.init,
        resources: app.resources,
        tasks: app.tasks
            .into_iter()
            .map(|(k, v)| {
                let v = ::check::task(&k.to_string(), v)?;

                Ok((k, v))
            })
            .collect::<Result<_>>()?,
    };

    Ok(app)
}

fn task(name: &str, task: syntax::check::Task) -> Result<Task> {
    let kind = match Exception::from(name) {
        Some(e) => {
            ensure!(
                task.enabled.is_none(),
                "`enabled` field is not valid for exceptions"
            );

            Kind::Exception(e)
        }
        None => Kind::Interrupt {
            enabled: task.enabled.unwrap_or(true),
        },
    };

    Ok(Task {
        kind,
        path: task.path,
        priority: task.priority.unwrap_or(1),
        resources: task.resources,
    })
}
