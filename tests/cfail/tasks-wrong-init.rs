// error-pattern: mismatched types

#![feature(used)]

#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use rtfm::{C1, P0, P1};
use device::interrupt::Exti0;

tasks!(device, {
    j1: Task {
        interrupt: Exti0,
        priority: P1,
        enabled: true,
    },
});

// WRONG. `init` must have signature `fn(P0, &C16)`
fn init(_: P0, _: &C1) {}

fn idle(_: P0) -> ! {
    loop {}
}

fn j1(_task: Exti0, _prio: P1) {}

// fake device crate
extern crate core;
extern crate cortex_m;

mod device {
    pub mod interrupt {
        use cortex_m::interrupt::Nr;

        extern "C" fn default_handler<T>(_: T) {}

        pub struct Handlers {
            pub Exti0: extern "C" fn(Exti0),
            pub Exti1: extern "C" fn(Exti1),
        }

        pub struct Exti0;
        pub struct Exti1;

        pub enum Interrupt {
            Exti0,
            Exti1,
        }

        unsafe impl Nr for Interrupt {
            fn nr(&self) -> u8 {
                0
            }
        }

        pub const DEFAULT_HANDLERS: Handlers = Handlers {
            Exti0: default_handler,
            Exti1: default_handler,
        };
    }
}
