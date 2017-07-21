//! An application with one task

#![deny(unsafe_code)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use cortex_m::peripheral::SystClkSource;
use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    // Here tasks are declared
    //
    // Each task corresponds to an interrupt or an exception. Every time the
    // interrupt or exception becomes *pending* the corresponding task handler
    // will be executed.
    tasks: {
        // Here we declare that we'll use the SYS_TICK exception as a task
        SYS_TICK: {
            // This is the priority of the task.
            // 1 is the lowest priority a task can have.
            // The maximum priority is determined by the number of priority bits
            // the device has. This device has 4 priority bits so 16 is the
            // maximum value.
            priority: 1,

            // These are the *resources* associated with this task
            //
            // The peripherals that the task needs can be listed here
            resources: [GPIOC],
        },
    }
}

fn init(p: init::Peripherals) {
    // power on GPIOC
    p.RCC.apb2enr.modify(|_, w| w.iopcen().enabled());

    // configure PC13 as output
    p.GPIOC.bsrr.write(|w| w.bs13().set());
    p.GPIOC
        .crh
        .modify(|_, w| w.mode13().output().cnf13().push());

    // configure the system timer to generate one interrupt every second
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000); // 1s
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

// This binds the `sys_tick` handler to the `SYS_TICK` task
//
// This particular handler has local state associated to it. The value of the
// `STATE` variable will be preserved across invocations of this handler
task!(SYS_TICK, sys_tick, Locals {
    static STATE: bool = false;
});

// This is the task handler of the SYS_TICK exception
//
// `t` is the preemption threshold token. We won't use it this time.
// `l` is the data local to this task. The type here must match the one declared
// in `task!`.
// `r` is the resources this task has access to. `SYS_TICK::Resources` has one
// field per resource declared in `app!`.
fn sys_tick(_t: &mut Threshold, l: &mut Locals, r: SYS_TICK::Resources) {
    // toggle state
    *l.STATE = !*l.STATE;

    if *l.STATE {
        // set the pin PC13 high
        r.GPIOC.bsrr.write(|w| w.bs13().set());
    } else {
        // set the pin PC13 low
        r.GPIOC.bsrr.write(|w| w.br13().reset());
    }
}
