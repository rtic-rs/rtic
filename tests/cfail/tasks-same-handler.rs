// error-pattern: specified more than once

#![feature(used)]

extern crate core;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_srp;

use cortex_m_srp::{C16, P0, P1, P2};
use device::interrupt::Exti0;

// WRONG: Two tasks mapped to the same interrupt handler
tasks!(device, {
    j1: (Exti0, P1),
    j2: (Exti0, P2),
});

fn init(_: C16) {}

fn idle(_: P0) {}

fn j1(_task: Exti0, _prio: P1) {}

fn j2(_task: Exti0, _prio: P1) {}


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
