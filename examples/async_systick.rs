//! examples/async_task2
#![no_main]
#![no_std]
#![feature(const_fn)]
#![feature(type_alias_impl_trait)]

// use core::cell::Cell;
// use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
// use core::mem::MaybeUninit;
use core::pin::Pin;
// use core::ptr;
// use core::ptr::NonNull;
// use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0], peripherals = true)]
mod app {
    use crate::Timer;
    use crate::*;

    #[resources]
    struct Resources {
        syst: cortex_m::peripheral::SYST,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        hprintln!("init").unwrap();
        foo::spawn().unwrap();
        init::LateResources { syst: cx.core.SYST }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // debug::exit(debug::EXIT_SUCCESS);
        loop {
            hprintln!("idle");
            cortex_m::asm::wfi(); // put the MCU in sleep mode until interrupt occurs
        }
    }

    #[task(resources = [syst])]
    fn foo(mut cx: foo::Context) {
        // BEGIN BOILERPLATE
        type F = impl Future + 'static;
        fn create(cx: foo::Context<'static>) -> F {
            task(cx)
        }

        static mut TASK: Task<F> = Task::new();

        hprintln!("foo trampoline").ok();
        unsafe {
            match TASK {
                Task::Idle | Task::Done(_) => {
                    hprintln!("foo spawn task").ok();
                    TASK.spawn(|| create(mem::transmute(cx)));
                }
                _ => {}
            };

            hprintln!("foo trampoline poll").ok();
            TASK.poll(|| {});

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

        async fn task(mut cx: foo::Context<'static>) {
            hprintln!("foo task").ok();

            hprintln!("delay long time").ok();
            let fut = cx.resources.syst.lock(|syst| timer_delay(syst, 5000000));

            hprintln!("we have just created the future");
            fut.await; // this calls poll on the timer future
            hprintln!("foo task resumed").ok();

            hprintln!("delay short time").ok();
            cx.resources
                .syst
                .lock(|syst| timer_delay(syst, 1000000))
                .await;
            hprintln!("foo task resumed").ok();
        }
    }

    // #[task(resources = [syst])]
    // fn timer(cx: timer::Context<'static>) {
    //     // BEGIN BOILERPLATE
    //     type F = impl Future + 'static;
    //     fn create(cx: timer::Context<'static>) -> F {
    //         task(cx)
    //     }

    //     static mut TASK: Task<F> = Task::new();

    //     hprintln!("timer trampoline").ok();
    //     unsafe {
    //         match TASK {
    //             Task::Idle | Task::Done(_) => {
    //                 hprintln!("create task").ok();
    //                 TASK.spawn(|| create(mem::transmute(cx)));
    //             }
    //             _ => {}
    //         };
    //         hprintln!("timer poll").ok();

    //         TASK.poll(|| {});

    //         match TASK {
    //             Task::Done(_) => {
    //                 hprintln!("timer done").ok();
    //             }
    //             _ => {
    //                 hprintln!("running").ok();
    //             }
    //         }
    //     }
    //     // END BOILERPLATE

    //     // for now assume this async task is done directly
    //     async fn task(mut cx: timer::Context<'static>) {
    //         hprintln!("SysTick").ok();

    //         Timer::delay(100000).await;

    //         // cx.resources.waker.lock(|w| *w = Some())
    //     }
    // }

    // This the actual RTIC task, binds to systic.
    #[task(binds = SysTick, resources = [syst], priority = 2)]
    fn systic(mut cx: systic::Context) {
        hprintln!("systic interrupt").ok();
        cx.resources.syst.lock(|syst| syst.disable_interrupt());
        crate::app::foo::spawn(); // this should be from a Queue later
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
