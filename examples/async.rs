#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0], peripherals = true)]
mod app {
    use crate::Timer;
    use crate::*;

    #[shared]
    struct Shared {
        syst: cortex_m::peripheral::SYST,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("init").unwrap();
        foo::spawn().unwrap();
        foo2::spawn().unwrap();
        (Shared { syst: cx.core.SYST }, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // debug::exit(debug::EXIT_SUCCESS);
        loop {
            hprintln!("idle");
            cortex_m::asm::wfi(); // put the MCU in sleep mode until interrupt occurs
        }
    }

    type F = impl Future + 'static;
    static mut TASK: Task<F> = Task::new();

    #[task(shared = [syst])]
    fn foo(mut cx: foo::Context) {
        // BEGIN BOILERPLATE
        fn create(cx: foo::Context<'static>) -> F {
            task(cx)
        }

        hprintln!("foo trampoline").ok();
        unsafe {
            match TASK {
                Task::Idle | Task::Done(_) => {
                    hprintln!("foo spawn task").ok();
                    TASK.spawn(|| create(mem::transmute(cx)));
                    // Per:
                    // I think transmute could be removed as in:
                    // TASK.spawn(|| create(cx));
                    //
                    // This could be done if spawn for async tasks would be passed
                    // a 'static reference by the generated code.
                    //
                    // Soundness:
                    // Check if lifetime for async context is correct.
                }
                _ => {}
            };

            foo_poll::spawn();
        }
        // END BOILERPLATE

        async fn task(mut cx: foo::Context<'static>) {
            hprintln!("foo task").ok();

            hprintln!("delay long time").ok();
            let fut = cx.shared.syst.lock(|syst| timer_delay(syst, 5000000));

            hprintln!("we have just created the future");
            fut.await; // this calls poll on the timer future
            hprintln!("foo task resumed").ok();

            hprintln!("delay short time").ok();
            cx.shared.syst.lock(|syst| timer_delay(syst, 1000000)).await;
            hprintln!("foo task resumed").ok();
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[task(shared = [syst])]
    fn foo_poll(mut cx: foo_poll::Context) {
        // BEGIN BOILERPLATE

        hprintln!("foo poll trampoline").ok();
        unsafe {
            hprintln!("foo trampoline poll").ok();
            TASK.poll(|| {
                hprintln!("foo poll closure").ok();
            });

            match TASK {
                Task::Done(ref r) => {
                    hprintln!("foo trampoline done").ok();
                    // hprintln!("r = {:?}", mem::transmute::<_, &u32>(r)).ok();
                }
                _ => {
                    hprintln!("foo trampoline running").ok();
                }
            }
        }
        // END BOILERPLATE
    }

    type F2 = impl Future + 'static;
    static mut TASK2: Task<F2> = Task::new();

    #[task(shared = [syst])]
    fn foo2(mut cx: foo2::Context) {
        // BEGIN BOILERPLATE
        fn create(cx: foo2::Context<'static>) -> F2 {
            task(cx)
        }

        hprintln!("foo2 trampoline").ok();
        unsafe {
            match TASK2 {
                Task::Idle | Task::Done(_) => {
                    hprintln!("foo2 spawn task").ok();
                    TASK2.spawn(|| create(mem::transmute(cx)));
                    // Per:
                    // I think transmute could be removed as in:
                    // TASK.spawn(|| create(cx));
                    //
                    // This could be done if spawn for async tasks would be passed
                    // a 'static reference by the generated code.
                    //
                    // Soundness:
                    // Check if lifetime for async context is correct.
                }
                _ => {}
            };

            foo2_poll::spawn();
        }
        // END BOILERPLATE

        async fn task(mut cx: foo2::Context<'static>) {
            hprintln!("foo2 task").ok();

            hprintln!("foo2 delay long time").ok();
            let fut = cx.shared.syst.lock(|syst| timer_delay(syst, 10_000_000));

            hprintln!("we have just created the future");
            fut.await; // this calls poll on the timer future
            hprintln!("foo task resumed").ok();
        }
    }

    #[task(shared = [syst])]
    fn foo2_poll(mut cx: foo2_poll::Context) {
        // BEGIN BOILERPLATE

        hprintln!("foo2 poll trampoline").ok();
        unsafe {
            hprintln!("foo2 trampoline poll").ok();
            TASK2.poll(|| {
                hprintln!("foo2 poll closure").ok();
            });

            match TASK2 {
                Task::Done(ref r) => {
                    hprintln!("foo2 trampoline done").ok();
                    // hprintln!("r = {:?}", mem::transmute::<_, &u32>(r)).ok();
                }
                _ => {
                    hprintln!("foo2 trampoline running").ok();
                }
            }
        }
        // END BOILERPLATE
    }

    // This the actual RTIC task, binds to systic.
    #[task(binds = SysTick, shared = [syst], priority = 2)]
    fn systic(mut cx: systic::Context) {
        hprintln!("systic interrupt").ok();
        cx.shared.syst.lock(|syst| syst.disable_interrupt());
        crate::app::foo_poll::spawn(); // this should be from a Queue later
        crate::app::foo2_poll::spawn(); // this should be from a Queue later
    }
}

//=============
// Waker

static WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake, waker_drop);

unsafe fn waker_clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &WAKER_VTABLE)
}

unsafe fn waker_wake(p: *const ()) {
    let f: fn() = mem::transmute(p);
    f();
}

unsafe fn waker_drop(_: *const ()) {
    // nop
}

//============
// Task

enum Task<F: Future + 'static> {
    Idle,
    Running(F),
    Done(F::Output),
}

impl<F: Future + 'static> Task<F> {
    const fn new() -> Self {
        Self::Idle
    }

    fn spawn(&mut self, future: impl FnOnce() -> F) {
        *self = Task::Running(future());
    }

    unsafe fn poll(&mut self, wake: fn()) {
        match self {
            Task::Idle => {}
            Task::Running(future) => {
                let future = Pin::new_unchecked(future);
                let waker_data: *const () = mem::transmute(wake);
                let waker = Waker::from_raw(RawWaker::new(waker_data, &WAKER_VTABLE));
                let mut cx = Context::from_waker(&waker);

                match future.poll(&mut cx) {
                    Poll::Ready(r) => *self = Task::Done(r),
                    Poll::Pending => {}
                };
            }
            Task::Done(_) => {}
        }
    }
}

//=============
// Timer
// Later we want a proper queue

use heapless;
pub struct Timer {
    pub done: bool,
    // pub waker_task: Option<fn() -> Result<(), ()>>,
}

impl Future for Timer {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.done {
            Poll::Ready(())
        } else {
            hprintln!("timer polled");
            cx.waker().wake_by_ref();
            hprintln!("after wake_by_ref");
            self.done = true;
            Poll::Pending
        }
    }
}

fn timer_delay(syst: &mut cortex_m::peripheral::SYST, t: u32) -> Timer {
    hprintln!("timer_delay {}", t);

    syst.set_reload(t);
    syst.enable_counter();
    syst.enable_interrupt();
    Timer {
        done: false,
        // waker_task: Some(app::foo::spawn), // we should add waker field to async task context i RTIC
    }
}
