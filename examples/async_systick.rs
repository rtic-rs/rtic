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
    use crate::*;

    #[resources]
    struct Resources {
        syst: cortex_m::peripheral::SYST,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // foo::spawn().unwrap();
        let mut syst = cx.core.SYST;
        syst.set_reload(100000);
        syst.enable_interrupt();
        syst.enable_counter();

        hprintln!("init").unwrap();

        init::LateResources { syst }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // foo::spawn().unwrap();
        // debug::exit(debug::EXIT_SUCCESS);
        loop {
            continue;
        }
    }

    #[task]
    fn foo(_c: foo::Context) {
        // BEGIN BOILERPLATE
        type F = impl Future + 'static;
        fn create() -> F {
            task()
        }

        static mut TASK: Task<F> = Task::new();

        hprintln!("foo trampoline").ok();
        unsafe {
            match TASK {
                Task::Idle | Task::Done(_) => {
                    hprintln!("create task").ok();
                    TASK.spawn(create);
                }
                _ => {}
            };
            hprintln!("poll").ok();

            TASK.poll(|| {});

            match TASK {
                Task::Done(ref r) => {
                    hprintln!("done").ok();
                    hprintln!("r = {:?}", mem::transmute::<_, &u32>(r)).ok();
                }
                _ => {
                    hprintln!("running").ok();
                }
            }
        }
        // END BOILERPLATE

        async fn task() -> u32 {
            hprintln!("foo1").ok();
            // let a: u32 = bar::spawn().await;
            // hprintln!("foo2 {}", a).ok();
            5
        }
    }

    #[task(resources = [syst])]
    fn timer(cx: timer::Context<'static>) {
        // BEGIN BOILERPLATE
        type F = impl Future + 'static;
        fn create(cx: timer::Context<'static>) -> F {
            task(cx)
        }

        static mut TASK: Task<F> = Task::new();

        hprintln!("timer trampoline").ok();
        unsafe {
            match TASK {
                Task::Idle | Task::Done(_) => {
                    hprintln!("create task").ok();
                    TASK.spawn(|| create(cx));
                }
                _ => {}
            };
            hprintln!("timer poll").ok();

            TASK.poll(|| {});

            match TASK {
                Task::Done(_) => {
                    hprintln!("timer done").ok();
                }
                _ => {
                    hprintln!("running").ok();
                }
            }
        }
        // END BOILERPLATE

        // for now assume this async task is done directly
        async fn task(cx: timer::Context<'static>) {
            hprintln!("SysTick ").ok();
        }
    }

    // This the actual RTIC task, binds to systic.
    #[task(binds = SysTick)]
    fn systic(cx: systic::Context) {}
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
// Yield

struct Yield {
    done: bool,
}

impl Future for Yield {
    type Output = u32;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.done {
            Poll::Ready(73)
        } else {
            cx.waker().wake_by_ref();
            self.done = true;
            Poll::Pending
        }
    }
}

fn please_yield() -> Yield {
    Yield { done: false }
}
