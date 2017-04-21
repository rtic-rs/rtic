// error-pattern: type mismatch

#![feature(used)]

extern crate core;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use rtfm::{C16, P0, P1};
use device::interrupt::Exti0;

/// Tasks can't have priority 0. Only idle has priority 0
tasks!(device, {
    j1: (Exti0, P0),
});

fn init(_: P0, _: &C16) {}

fn idle(_: P0) -> ! {
    loop {}
}

fn j1(_task: Exti0, _prio: P1) {}

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
