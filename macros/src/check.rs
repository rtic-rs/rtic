use std::collections::HashSet;

use proc_macro2::Span;
use rtfm_syntax::{
    analyze::Analysis,
    ast::{App, CustomArg},
};
use syn::{parse, Path};

pub struct Extra<'a> {
    pub device: &'a Path,
    pub monotonic: Option<&'a Path>,
    pub peripherals: Option<u8>,
}

impl<'a> Extra<'a> {
    pub fn monotonic(&self) -> &'a Path {
        self.monotonic.expect("UNREACHABLE")
    }
}

pub fn app<'a>(app: &'a App, analysis: &Analysis) -> parse::Result<Extra<'a>> {
    if cfg!(feature = "homogeneous") {
        // this RTFM mode uses the same namespace for all cores so we need to check that the
        // identifiers used for each core `#[init]` and `#[idle]` functions don't collide
        let mut seen = HashSet::new();

        for name in app
            .inits
            .values()
            .map(|init| &init.name)
            .chain(app.idles.values().map(|idle| &idle.name))
        {
            if seen.contains(name) {
                return Err(parse::Error::new(
                    name.span(),
                    "this identifier is already being used by another core",
                ));
            } else {
                seen.insert(name);
            }
        }
    }

    // check that all exceptions are valid; only exceptions with configurable priorities are
    // accepted
    for (name, task) in &app.hardware_tasks {
        let name_s = task.args.binds.to_string();
        match &*name_s {
            "SysTick" => {
                if analysis.timer_queues.get(&task.args.core).is_some() {
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
    for name in app
        .extern_interrupts
        .iter()
        .flat_map(|(_, interrupts)| interrupts.keys())
    {
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
    for core in 0..app.args.cores {
        let mut first = None;
        let priorities = app
            .software_tasks
            .iter()
            .filter_map(|(name, task)| {
                if task.args.core == core {
                    first = Some(name);
                    Some(task.args.priority)
                } else {
                    None
                }
            })
            .chain(analysis.timer_queues.get(&core).map(|tq| tq.priority))
            .collect::<HashSet<_>>();

        let need = priorities.len();
        let given = app
            .extern_interrupts
            .get(&core)
            .map(|ei| ei.len())
            .unwrap_or(0);
        if need > given {
            let s = if app.args.cores == 1 {
                format!(
                    "not enough `extern` interrupts to dispatch \
                     all software tasks (need: {}; given: {})",
                    need, given
                )
            } else {
                format!(
                    "not enough `extern` interrupts to dispatch \
                     all software tasks on this core (need: {}; given: {})",
                    need, given
                )
            };

            return Err(parse::Error::new(first.unwrap().span(), &s));
        }
    }

    let mut device = None;
    let mut monotonic = None;
    let mut peripherals = None;

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
                CustomArg::Bool(x) if app.args.cores == 1 => {
                    peripherals = if *x { Some(0) } else { None }
                }

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

                _ => {
                    return Err(parse::Error::new(
                        k.span(),
                        if app.args.cores == 1 {
                            "unexpected argument value; this should be a boolean"
                        } else {
                            "unexpected argument value; this should be an integer"
                        },
                    ));
                }
            },

            _ => {
                return Err(parse::Error::new(k.span(), "unexpected argument"));
            }
        }
    }

    if !analysis.timer_queues.is_empty() && monotonic.is_none() {
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
            "a `device` argument must be specified in `#[rtfm::app]`",
        ))
    }
}
