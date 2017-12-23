//! Demonstrates initialization of resources in `init`.
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        // Usually, resources are initialized with a constant initializer:
        static ON: bool = false;

        // However, there are cases where this is not possible or not desired.
        // For example, there may not be a sensible value to use, or the type may
        // not be constructible in a constant (like `Vec`).
        //
        // While it is possible to use an `Option` in some cases, that requires
        // you to properly initialize it and `.unwrap()` it at every use. It
        // also consumes more memory.
        //
        // To solve this, it is possible to defer initialization of resources to
        // `init` by omitting the initializer. Doing that will require `init` to
        // return the values of all "late" resources.
        static IP_ADDRESS: u32;

        // PORT is used by 2 tasks, making it a shared resource. This just tests
        // another internal code path and is not important for the example.
        static PORT: u16;
    },

    idle: {
        // Test that late resources can be used in idle
        resources: [IP_ADDRESS],
    },

    tasks: {
        SYS_TICK: {
            priority: 1,
            path: sys_tick,
            resources: [IP_ADDRESS, PORT, ON],
        },

        EXTI0: {
            priority: 2,
            path: exti0,
            resources: [PORT],
        }
    }
}

// The signature of `init` is now required to have a specific return type.
fn init(_p: init::Peripherals, _r: init::Resources) -> init::LateResources {
    // `init::Resources` does not contain `IP_ADDRESS`, since it is not yet
    // initialized.
    //_r.IP_ADDRESS;     // doesn't compile

    // ...obtain value for IP_ADDRESS from EEPROM/DHCP...
    let ip_address = 0x7f000001;

    init::LateResources {
        // This struct will contain fields for all resources with omitted
        // initializers.
        IP_ADDRESS: ip_address,
        PORT: 0,
    }
}

fn sys_tick(_t: &mut Threshold, r: SYS_TICK::Resources) {
    // Other tasks can access late resources like any other, since they are
    // guaranteed to be initialized when tasks are run.

    r.IP_ADDRESS;
}

fn exti0(_t: &mut Threshold, _r: EXTI0::Resources) {}

fn idle(_t: &mut Threshold, _r: idle::Resources) -> ! {
    loop {
        rtfm::wfi();
    }
}
