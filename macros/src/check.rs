use std::collections::HashSet;

use crate::syntax::ast::App;
use syn::parse;

pub fn app(app: &App) -> parse::Result<()> {
    // Check that external (device-specific) interrupts are not named after known (Cortex-M)
    // exceptions
    for name in app.args.dispatchers.keys() {
        let name_s = name.to_string();

        match &*name_s {
            "NonMaskableInt" | "HardFault" | "MemoryManagement" | "BusFault" | "UsageFault"
            | "SecureFault" | "SVCall" | "DebugMonitor" | "PendSV" | "SysTick" => {
                return Err(parse::Error::new(
                    name.span(),
                    "Cortex-M exceptions can't be used as `extern` interrupts",
                ));
            }

            _ => {}
        }
    }

    // Check that there are enough external interrupts to dispatch the software tasks and the timer
    // queue handler
    let mut first = None;
    let priorities = app
        .software_tasks
        .iter()
        .map(|(name, task)| {
            first = Some(name);
            task.args.priority
        })
        .filter(|prio| *prio > 0)
        .collect::<HashSet<_>>();

    let need = priorities.len();
    let given = app.args.dispatchers.len();
    if need > given {
        let s = {
            format!(
                "not enough interrupts to dispatch \
                    all software tasks (need: {need}; given: {given})"
            )
        };

        // If not enough tasks and first still is None, may cause
        // "custom attribute panicked" due to unwrap on None
        return Err(parse::Error::new(first.unwrap().span(), s));
    }

    // Check that all exceptions are valid; only exceptions with configurable priorities are
    // accepted
    for (name, task) in &app.hardware_tasks {
        let name_s = task.args.binds.to_string();
        match &*name_s {
            "NonMaskableInt" | "HardFault" => {
                return Err(parse::Error::new(
                    name.span(),
                    "only exceptions with configurable priority can be used as hardware tasks",
                ));
            }

            _ => {}
        }
    }

    Ok(())
}
