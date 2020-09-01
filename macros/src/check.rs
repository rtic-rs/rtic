use std::collections::HashSet;

use proc_macro2::Span;
use rtic_syntax::{
    analyze::Analysis,
    ast::{App, CustomArg},
};
use syn::{parse, Path};

pub struct Extra<'a> {
    pub device: &'a Path,
    pub monotonic: Option<&'a Path>,
    pub peripherals: bool,
}

impl<'a> Extra<'a> {
    pub fn monotonic(&self) -> &'a Path {
        self.monotonic.expect("UNREACHABLE")
    }
}

pub fn app<'a>(app: &'a App, analysis: &Analysis) -> parse::Result<Extra<'a>> {
    // check that all exceptions are valid; only exceptions with configurable priorities are
    // accepted
    for (name, task) in &app.hardware_tasks {
        let name_s = task.args.binds.to_string();
        match &*name_s {
            "SysTick" => {
                // If the timer queue is used, then SysTick is unavailable
                if !analysis.timer_queues.is_empty() {
                    return Err(parse::Error::new(
                        name.span(),
                        "this exception can't be used because it's being used by the runtime",
                    ));
                } else {
                    // OK
                }
            }

            "NonMaskableInt" | "HardFault" => {
                return Err(parse::Error::new(
                    name.span(),
                    "only exceptions with configurable priority can be used as hardware tasks",
                ));
            }

            _ => {}
        }
    }

    // check that external (device-specific) interrupts are not named after known (Cortex-M)
    // exceptions
    for name in app.extern_interrupts.keys() {
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

    // check that there are enough external interrupts to dispatch the software tasks and the timer
    // queue handler
    let mut first = None;
    let priorities = app
        .software_tasks
        .iter()
        .filter_map(|(name, task)| {
            first = Some(name);
            Some(task.args.priority)
        })
        .chain(analysis.timer_queues.first().map(|tq| tq.priority))
        .collect::<HashSet<_>>();

    let need = priorities.len();
    let given = app.extern_interrupts.len();
    if need > given {
        let s = {
            format!(
                "not enough `extern` interrupts to dispatch \
                    all software tasks (need: {}; given: {})",
                need, given
            )
        };

        // If not enough tasks and first still is None, may cause
        // "custom attribute panicked"
        // unwrap on None
        return Err(parse::Error::new(first.unwrap().span(), &s));
    }

    let mut device = None;
    let mut monotonic = None;
    let mut peripherals = false;

    for (k, v) in &app.args.custom {
        let ks = k.to_string();

        match &*ks {
            "device" => match v {
                CustomArg::Path(p) => device = Some(p),

                _ => {
                    return Err(parse::Error::new(
                        k.span(),
                        "unexpected argument value; this should be a path",
                    ));
                }
            },

            "monotonic" => match v {
                CustomArg::Path(p) => monotonic = Some(p),

                _ => {
                    return Err(parse::Error::new(
                        k.span(),
                        "unexpected argument value; this should be a path",
                    ));
                }
            },

            "peripherals" => match v {
                CustomArg::Bool(x) => peripherals = if *x { true } else { false },

                /*
                CustomArg::UInt(s) if app.args.cores != 1 => {
                    let x = s.parse::<u8>().ok();
                    peripherals = if x.is_some() && x.unwrap() < app.args.cores {
                        Some(x.unwrap())
                    } else {
                        return Err(parse::Error::new(
                            k.span(),
                            &format!(
                                "unexpected argument value; \
                                 this should be an integer in the range 0..={}",
                                app.args.cores
                            ),
                        ));
                    }
                }
                */
                _ => {
                    return Err(parse::Error::new(
                        k.span(),
                        //if app.args.cores == 1 {
                        "unexpected argument value; this should be a boolean",
                        /*
                        } else {
                            "unexpected argument value; this should be an integer"
                        },
                            */
                    ));
                }
            },

            _ => {
                return Err(parse::Error::new(k.span(), "unexpected argument"));
            }
        }
    }

    if !&analysis.timer_queues.is_empty() && monotonic.is_none() {
        return Err(parse::Error::new(
            Span::call_site(),
            "a `monotonic` timer must be specified to use the `schedule` API",
        ));
    }

    if let Some(device) = device {
        Ok(Extra {
            device,
            monotonic,
            peripherals,
        })
    } else {
        Err(parse::Error::new(
            Span::call_site(),
            "a `device` argument must be specified in `#[rtic::app]`",
        ))
    }
}
