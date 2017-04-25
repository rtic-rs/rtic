// error-pattern: symbol `GPIOA` is already defined

#![feature(const_fn)]
#![feature(used)]

#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use rtfm::{C16, P0, P1};
use device::interrupt::Exti0;

peripherals!(device, {
    GPIOA: Peripheral {
        register_block: Gpioa,
        ceiling: C1,
    },
});

mod foo {
    // WRONG: peripheral alias
    peripherals!(device, {
        GPIOA: Peripheral {
            register_block: Gpioa,
            ceiling: C2,
        },
    });
}

tasks!(device, {});

fn init(_: P0, _: &C16) {}

fn idle(_: P0) -> ! {
    loop {}
}

fn j1(_task: Exti0, _prio: P1) {}

// fake device crate
extern crate core;
extern crate cortex_m;

mod device {
    use cortex_m::peripheral::Peripheral;

    pub const GPIOA: Peripheral<Gpioa> = unsafe { Peripheral::new(0x0) };

    pub struct Gpioa;

    pub mod interrupt {
        use cortex_m::interrupt::Nr;

        extern "C" fn default_handler<T>(_: T) {}

        pub struct Handlers {
            pub Exti0: extern "C" fn(Exti0),
        }

        pub struct Exti0;

        pub enum Interrupt {
            Exti0,
        }

        unsafe impl Nr for Interrupt {
            fn nr(&self) -> u8 {
                0
            }
        }

        pub const DEFAULT_HANDLERS: Handlers =
            Handlers { Exti0: default_handler };
    }
}
