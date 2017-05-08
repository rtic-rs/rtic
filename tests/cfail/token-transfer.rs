#![feature(const_fn)]
#![feature(optin_builtin_traits)]
#![feature(used)]

#[macro_use]
extern crate cortex_m_rtfm as rtfm;

use core::cell::RefCell;

use rtfm::{C2, Local, P0, P1, P2, Resource, T0, T1, T2, TMax};
use device::interrupt::{Exti0, Exti1};

tasks!(device, {
    t1: Task {
        interrupt: Exti0,
        priority: P1,
        enabled: true,
    },
    t2: Task {
        interrupt: Exti1,
        priority: P2,
        enabled: true,
    },
});

fn init(_: P0, _: &TMax) {}

fn idle(_: P0, _: T0) -> ! {
    rtfm::request(t1);
    rtfm::request(t1);

    loop {}
}

static CHANNEL: Resource<RefCell<Option<Exti0>>, C2> = {
    //~^ error: Send
    Resource::new(RefCell::new(None))
};

static LOCAL: Local<i32, Exti0> = Local::new(0);

fn t1(mut task: Exti0, ref priority: P1, ref threshold: T1) {
    // First run
    static FIRST: Local<bool, Exti0> = Local::new(true);

    let first = *FIRST.borrow(&task);

    if first {
        // toggle
        *FIRST.borrow_mut(&mut task) = false;
    }

    if first {
        threshold.raise(
            &CHANNEL, move |threshold| {
                let channel = CHANNEL.access(priority, threshold);

                // BAD: give up task token
                *channel.borrow_mut() = Some(task);
            }
        );

        return;
    }

    let _local = LOCAL.borrow_mut(&mut task);

    // ..

    // `t2` will preempt `t1`
    rtfm::request(t2);

    // ..

    // `LOCAL` mutably borrowed up to this point
}

fn t2(_task: Exti1, ref priority: P2, ref threshold: T2) {
    let channel = CHANNEL.access(priority, threshold);
    let mut channel = channel.borrow_mut();

    if let Some(mut other_task) = channel.take() {
        // BAD: `t2` has access to `t1`'s task token
        // so it can now mutably access local while `t1` is also using it
        let _local = LOCAL.borrow_mut(&mut other_task);

    }
}

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

        impl !Send for Exti0 {}

        unsafe impl Context for Exti1 {}

        unsafe impl Nr for Exti1 {
            fn nr(&self) -> u8 {
                0
            }
        }

        impl !Send for Exti1 {}

        unsafe impl Context for Exti2 {}

        unsafe impl Nr for Exti2 {
            fn nr(&self) -> u8 {
                0
            }
        }

        impl !Send for Exti2 {}

        pub const DEFAULT_HANDLERS: Handlers = Handlers {
            Exti0: default_handler,
            Exti1: default_handler,
            Exti2: default_handler,
        };
    }
}
