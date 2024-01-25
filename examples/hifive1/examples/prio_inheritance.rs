#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

// use panic_halt as _;
use riscv_rt as _;

#[rtic::app(device = e310x, backend = HART0)]
mod app {
    use core::{future::Future, pin::Pin, task::Context, task::Poll};
    use hifive1::hal::prelude::*;
    use semihosting::{println, process::exit};

    /// Dummy asynchronous function to showcase SW tasks
    pub async fn yield_now(task: &str) {
        /// Yield implementation
        struct YieldNow {
            yielded: bool,
        }
        println!("  [{}]: Yield", task);

        impl Future for YieldNow {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.yielded {
                    return Poll::Ready(());
                }

                self.yielded = true;
                cx.waker().wake_by_ref();

                Poll::Pending
            }
        }

        YieldNow { yielded: false }.await
    }

    #[shared]
    struct Shared {
        counter: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        // Pends the SoftLow interrupt but its handler won't run until *after*
        // `init` returns because interrupts are disabled
        let resources = unsafe { hifive1::hal::DeviceResources::steal() };
        let peripherals = resources.peripherals;

        let clocks =
            hifive1::configure_clocks(peripherals.PRCI, peripherals.AONCLK, 64.mhz().into());
        let gpio = resources.pins;

        // Configure UART for stdout
        hifive1::stdout::configure(
            peripherals.UART0,
            hifive1::pin!(gpio, uart0_tx),
            hifive1::pin!(gpio, uart0_rx),
            115_200.bps(),
            clocks,
        );

        (Shared { counter: 0 }, Local {})
    }

    #[idle(shared = [counter])]
    fn idle(mut cx: idle::Context) -> ! {
        println!("[Idle]: Started");
        // pend the medium priority SW task only once
        soft_medium::spawn().unwrap();
        cx.shared.counter.lock(|counter| {
            println!("[Idle]: Shared: {}", *counter);
        });
        // exit QEMU simulator
        println!("[Idle]: Finished");
        exit(0);
    }

    /// Medium priority SW task. It is triggered by the idle and spawns the rest of the SW tasks
    #[task(shared = [counter], priority = 2)]
    async fn soft_medium(mut cx: soft_medium::Context) {
        // Safe access to local `static mut` variable
        println!("    [SoftMedium]: Started");
        cx.shared.counter.lock(|counter| {
            // Spawn the other SW tasks INSIDE the critical section (just for showing priority inheritance)
            soft_low_1::spawn().unwrap();
            soft_high::spawn().unwrap();
            soft_low_2::spawn().unwrap();

            *counter += 1;
            println!("    [SoftMedium]: Shared: {}", *counter);
        });
        println!("    [SoftMedium]: Finished");
    }

    /// Low priority SW task. It runs cooperatively with soft_low_2
    #[task(shared = [counter], priority = 1)]
    async fn soft_low_1(mut cx: soft_low_1::Context) {
        println!("  [SoftLow1]: Started");
        cx.shared.counter.lock(|counter| {
            *counter += 1;
            println!("  [SoftLow1]: Shared: {}", *counter);
        });
        // Yield to the other SW task
        yield_now("SoftLow1").await;

        println!("  [SoftLow1]: Finished");
    }

    /// Low priority SW task. It runs cooperatively with soft_low_2
    #[task(shared = [counter], priority = 1)]
    async fn soft_low_2(mut cx: soft_low_2::Context) {
        println!("  [SoftLow2]: Started");
        cx.shared.counter.lock(|counter| {
            *counter += 1;
            println!("  [SoftLow2]: Shared: {}", *counter);
        });

        // Yield to the other SW task
        yield_now("SoftLow2").await;

        println!("  [SoftLow2]: Finished");
    }

    /// High priority SW task
    #[task(shared = [counter], priority = 3)]
    async fn soft_high(mut cx: soft_high::Context) {
        println!("      [SoftHigh]: Started");

        cx.shared.counter.lock(|counter| {
            *counter += 1;
            println!("      [SoftHigh]: Shared: {}", counter);
        });

        println!("      [SoftHigh]: Finished");
    }
}
