//! examples/common.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [SSI0, QEI0])]
mod app {
    use examples_runner::{println, exit};
    use systick_monotonic::*; // Implements the `Monotonic` trait

    // A monotonic timer to enable scheduling in RTIC
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

    // Resources shared between tasks
    #[shared]
    struct Shared {
        s1: u32,
        s2: i32,
    }

    // Local resources to specific tasks (cannot be shared)
    #[local]
    struct Local {
        l1: u8,
        l2: i8,
    }

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Spawn the task `foo` directly after `init` finishes
        foo::spawn().unwrap();

        // Spawn the task `bar` 1 second after `init` finishes, this is enabled
        // by the `#[monotonic(..)]` above
        bar::spawn_after(1.secs()).unwrap();

        exit();

        // (
        //     // Initialization of shared resources
        //     Shared { s1: 0, s2: 1 },
        //     // Initialization of task local resources
        //     Local { l1: 2, l2: 3 },
        //     // Move the monotonic timer to the RTIC run-time, this enables
        //     // scheduling
        //     init::Monotonics(mono),
        // )
    }

    // Background task, runs whenever no other tasks are running
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    // Software task, not bound to a hardware interrupt.
    // This task takes the task local resource `l1`
    // The resources `s1` and `s2` are shared between all other tasks.
    #[task(shared = [s1, s2], local = [l1])]
    fn foo(_: foo::Context) {
        // This task is only spawned once in `init`, hence this task will run
        // only once

        println!("foo");
    }

    // Software task, also not bound to a hardware interrupt
    // This task takes the task local resource `l2`
    // The resources `s1` and `s2` are shared between all other tasks.
    #[task(shared = [s1, s2], local = [l2])]
    fn bar(_: bar::Context) {
        println!("bar");

        // Run `bar` once per second
        bar::spawn_after(1.secs()).unwrap();
    }

    // Hardware task, bound to a hardware interrupt
    // The resources `s1` and `s2` are shared between all other tasks.
    #[task(binds = UART0, priority = 3, shared = [s1, s2])]
    fn uart0_interrupt(_: uart0_interrupt::Context) {
        // This task is bound to the interrupt `UART0` and will run
        // whenever the interrupt fires

        // Note that RTIC does NOT clear the interrupt flag, this is up to the
        // user

        println!("UART0 interrupt!");
    }
}
