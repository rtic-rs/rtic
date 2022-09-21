use std::collections::HashSet;

use proc_macro2::Span;
use rtic_syntax::{analyze::Analysis, ast::App};
use syn::{parse, Path};

pub struct Extra {
    pub device: Path,
    pub peripherals: bool,
}

pub fn app(app: &App, _analysis: &Analysis) -> parse::Result<Extra> {
    // Check that external (device-specific) interrupts are not named after known (Cortex-M)
    // exceptions
    for name in app.args.extern_interrupts.keys() {
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
            (task.args.priority, task.is_async)
        })
        .collect::<HashSet<_>>();

    let need_sync = priorities
        .iter()
        // Only count if not 0 and not async
        .filter(|(prio, is_async)| *prio > 0 && !*is_async)
        .count();

    let need_async = priorities
        .iter()
        // Only count if not 0 and async
        .filter(|(prio, is_async)| *prio > 0 && *is_async)
        .count();

    let given = app.args.extern_interrupts.len();
    if need_sync + need_async > given {
        let s = {
            format!(
                "not enough interrupts to dispatch all software and async tasks \
                 (need: {}; given: {}) - one interrupt is needed per priority and sync/async task",
                need_sync + need_async,
                given
            )
        };

        // If not enough tasks and first still is None, may cause
        // "custom attribute panicked" due to unwrap on None
        return Err(parse::Error::new(
            first.expect("RTIC-ICE: needed async + needed sync").span(),
            &s,
        ));
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

    if let Some(device) = app.args.device.clone() {
        Ok(Extra {
            device,
            peripherals: app.args.peripherals,
        })
    } else {
        Err(parse::Error::new(
            Span::call_site(),
            "a `device` argument must be specified in `#[rtic::app]`",
        ))
    }
}
