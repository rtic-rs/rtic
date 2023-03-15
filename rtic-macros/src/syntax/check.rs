use std::collections::HashSet;

use syn::parse;

use crate::syntax::ast::App;

pub fn app(app: &App) -> parse::Result<()> {
    // Check that all referenced resources have been declared
    // Check that resources are NOT `Exclusive`-ly shared
    let mut owners = HashSet::new();
    for (_, name, access) in app.shared_resource_accesses() {
        if app.shared_resources.get(name).is_none() {
            return Err(parse::Error::new(
                name.span(),
                "this shared resource has NOT been declared",
            ));
        }

        if access.is_exclusive() {
            owners.insert(name);
        }
    }

    for name in app.local_resource_accesses() {
        if app.local_resources.get(name).is_none() {
            return Err(parse::Error::new(
                name.span(),
                "this local resource has NOT been declared",
            ));
        }
    }

    // Check that no resource has both types of access (`Exclusive` & `Shared`)
    let exclusive_accesses = app
        .shared_resource_accesses()
        .filter_map(|(priority, name, access)| {
            if priority.is_some() && access.is_exclusive() {
                Some(name)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();
    for (_, name, access) in app.shared_resource_accesses() {
        if access.is_shared() && exclusive_accesses.contains(name) {
            return Err(parse::Error::new(
                name.span(),
                "this implementation doesn't support shared (`&-`) - exclusive (`&mut-`) locks; use `x` instead of `&x`",
            ));
        }
    }

    // check that dispatchers are not used as hardware tasks
    for task in app.hardware_tasks.values() {
        let binds = &task.args.binds;

        if app.args.dispatchers.contains_key(binds) {
            return Err(parse::Error::new(
                binds.span(),
                "dispatcher interrupts can't be used as hardware tasks",
            ));
        }
    }

    Ok(())
}
