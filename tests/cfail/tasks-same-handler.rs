// error-pattern: field `Exti0` specified more than once

#![feature(used)]

#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use rtfm::{C0, C1, C16, C2, P0, P1, P2};
use device::interrupt::{Exti0, Exti1};

// WRONG: Two tasks mapped to the same interrupt handler
tasks!(device, {
    j1: Task {
        interrupt: Exti0,
        priority: P1,
        enabled: true,
    },
    j2: Task {
        interrupt: Exti0,
        priority: P2,
        enabled: true,
    },
});

fn init(_: P0, _: &C16) {}

fn idle(_: P0, _: C0) -> ! {
    loop {}
}

fn j1(_task: Exti0, _prio: P1, _ceil: C1) {}

fn j2(_task: Exti0, _prio: P2, _ceil: C2) {}

// fake device crate
extern crate core;
extern crate cortex_m;

mod device {
    pub mod interrupt {
        use cortex_m::ctxt::Context;
        use cortex_m::interrupt::Nr;

        extern "C" fn default_handler<T>(_: T) {}

        pub struct Handlers {
            pub Exti0: extern "C" fn(Exti0),
            pub Exti1: extern "C" fn(Exti1),
            pub Exti2: extern "C" fn(Exti2),
        }

        pub struct Exti0;
        pub struct Exti1;
        pub struct Exti2;

        pub enum Interrupt {
            Exti0,
            Exti1,
            Exti2,
        }

        unsafe impl Nr for Interrupt {
            fn nr(&self) -> u8 {
                0
            }
        }

        unsafe impl Context for Exti0 {}

        unsafe impl Nr for Exti0 {
            fn nr(&self) -> u8 {
                0
            }
        }

        unsafe impl Context for Exti1 {}

        unsafe impl Nr for Exti1 {
            fn nr(&self) -> u8 {
                0
            }
        }

        unsafe impl Context for Exti2 {}

        unsafe impl Nr for Exti2 {
            fn nr(&self) -> u8 {
                0
            }
        }

        pub const DEFAULT_HANDLERS: Handlers = Handlers {
            Exti0: default_handler,
            Exti1: default_handler,
            Exti2: default_handler,
        };
    }
}
