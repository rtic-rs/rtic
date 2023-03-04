use syn::Ident;

use crate::syntax::{
    analyze::Priority,
    ast::{Access, App, Local, TaskLocal},
};

impl App {
    pub(crate) fn shared_resource_accesses(
        &self,
    ) -> impl Iterator<Item = (Option<Priority>, &Ident, Access)> {
        self.idle
            .iter()
            .flat_map(|idle| {
                idle.args
                    .shared_resources
                    .iter()
                    .map(move |(name, access)| (Some(0), name, *access))
            })
            .chain(self.hardware_tasks.values().flat_map(|task| {
                task.args
                    .shared_resources
                    .iter()
                    .map(move |(name, access)| (Some(task.args.priority), name, *access))
            }))
            .chain(self.software_tasks.values().flat_map(|task| {
                task.args
                    .shared_resources
                    .iter()
                    .map(move |(name, access)| (Some(task.args.priority), name, *access))
            }))
    }

    fn is_external(task_local: &TaskLocal) -> bool {
        matches!(task_local, TaskLocal::External)
    }

    pub(crate) fn local_resource_accesses(&self) -> impl Iterator<Item = &Ident> {
        self.init
            .args
            .local_resources
            .iter()
            .filter(|(_, task_local)| Self::is_external(task_local)) // Only check the resources declared in `#[local]`
            .map(move |(name, _)| name)
            .chain(self.idle.iter().flat_map(|idle| {
                idle.args
                    .local_resources
                    .iter()
                    .filter(|(_, task_local)| Self::is_external(task_local)) // Only check the resources declared in `#[local]`
                    .map(move |(name, _)| name)
            }))
            .chain(self.hardware_tasks.values().flat_map(|task| {
                task.args
                    .local_resources
                    .iter()
                    .filter(|(_, task_local)| Self::is_external(task_local)) // Only check the resources declared in `#[local]`
                    .map(move |(name, _)| name)
            }))
            .chain(self.software_tasks.values().flat_map(|task| {
                task.args
                    .local_resources
                    .iter()
                    .filter(|(_, task_local)| Self::is_external(task_local)) // Only check the resources declared in `#[local]`
                    .map(move |(name, _)| name)
            }))
    }

    fn get_declared_local(tl: &TaskLocal) -> Option<&Local> {
        match tl {
            TaskLocal::External => None,
            TaskLocal::Declared(l) => Some(l),
        }
    }

    /// Get all declared local resources, i.e. `local = [NAME: TYPE = EXPR]`.
    ///
    /// Returns a vector of (task name, resource name, `Local` struct)
    pub fn declared_local_resources(&self) -> Vec<(&Ident, &Ident, &Local)> {
        self.init
            .args
            .local_resources
            .iter()
            .filter_map(move |(name, tl)| {
                Self::get_declared_local(tl).map(|l| (&self.init.name, name, l))
            })
            .chain(self.idle.iter().flat_map(|idle| {
                idle.args
                    .local_resources
                    .iter()
                    .filter_map(move |(name, tl)| {
                        Self::get_declared_local(tl)
                            .map(|l| (&self.idle.as_ref().unwrap().name, name, l))
                    })
            }))
            .chain(self.hardware_tasks.iter().flat_map(|(task_name, task)| {
                task.args
                    .local_resources
                    .iter()
                    .filter_map(move |(name, tl)| {
                        Self::get_declared_local(tl).map(|l| (task_name, name, l))
                    })
            }))
            .chain(self.software_tasks.iter().flat_map(|(task_name, task)| {
                task.args
                    .local_resources
                    .iter()
                    .filter_map(move |(name, tl)| {
                        Self::get_declared_local(tl).map(|l| (task_name, name, l))
                    })
            }))
            .collect()
    }
}
