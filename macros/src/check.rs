use std::collections::HashMap;

use syn::{Ident, Path};
use syntax::check::{self, Idle, Init};
use syntax::{self, Idents, Statics};

use syntax::error::*;

pub struct App {
    pub device: Path,
    pub idle: Idle,
    pub init: Init,
    pub resources: Statics,
    pub tasks: Tasks,
}

pub type Tasks = HashMap<Ident, Task>;

pub struct Task {
    pub enabled: Option<bool>,
    pub path: Option<Path>,
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
            .map(|(k, v)| (k, ::check::task(v)))
            .collect(),
    };

    ::check::resources(&app)
        .chain_err(|| "checking `resources`")?;

    Ok(app)
}

fn resources(app: &App) -> Result<()> {
    for resource in app.resources.keys() {
        if app.idle.resources.contains(resource) {
            continue;
        }

        if app.tasks
            .values()
            .any(|task| task.resources.contains(resource))
        {
            continue;
        }

        bail!("resource `{}` is unused", resource);
    }

    Ok(())
}

fn task(task: syntax::check::Task) -> Task {
    Task {
        enabled: task.enabled,
        path: task.path,
        priority: task.priority.unwrap_or(1),
        resources: task.resources,
    }
}
