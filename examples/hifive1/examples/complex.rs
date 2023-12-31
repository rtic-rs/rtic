#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_halt as _;
use riscv_rt as _;

#[rtic::app(device = e310x, backend = HART0)]
mod app {
    use riscv_semihosting::hprintln;
    use core::{future::Future, pin::Pin, task::Context, task::Poll};
    use hifive1::{hal::prelude::*};

    /// Yield implementation for SW tasks
    pub async fn yield_now(task: &str) {
        /// Yield implementation
        struct YieldNow {
            yielded: bool,
        }
        hprintln!("[Yield]: {} is yielding", task);

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

    /// HW handler for MachineTimer interrupts triggered by CLINT.
    /// It also pends the middle priority SW task.
    #[no_mangle]
    #[allow(non_snake_case)]
    unsafe fn MachineTimer() {
        // increase mtimecmp to clear interrupt
        let mtimecmp = e310x::CLINT::mtimecmp0();
        let val = mtimecmp.read();
        hprintln!("--- update MTIMECMP (mtimecmp = {}) ---", val);
        mtimecmp.write(val + e310x::CLINT::freq() as u64);
        // we also pend the lowest priority SW task before the RTC SW task is automatically pended
        soft_medium::spawn().unwrap();
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

        hprintln!("Configuring CLINT...");
        e310x::CLINT::disable();
        let mtimer = e310x::CLINT::mtimer();
        mtimer.mtimecmp0.write(e310x::CLINT::freq() as u64);
        mtimer.mtime.write(0);
        unsafe {
            riscv_slic::set_interrupts();
            e310x::CLINT::mtimer_enable();
            riscv_slic::enable();
        }
        hprintln!("... done!");
        (Shared { counter: 0 }, Local {})
    }

    // The idle task is executed when no other task is running.
    // It is responsible for putting the CPU to sleep if there is nothing else to do.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            unsafe { riscv::asm::wfi() }; // wait for interruption
        }
    }

    /// Medium priority SW task. It is triggered by the CLINT timer interrupt, and spawns the rest of the SW tasks
    #[task(local = [times: u32 = 0], shared = [counter], priority = 2)]
    async fn soft_medium(mut cx: soft_medium::Context) {
        // Safe access to local `static mut` variable
        hprintln!("    [SoftMedium]: Started");
        cx.shared.counter.lock(|counter| {
            // Spawn the other SW tasks INSIDE the critical section (just for testing)
            soft_low_1::spawn().unwrap();
            soft_high::spawn().unwrap();
            soft_low_2::spawn().unwrap();

            *counter += 1;
            hprintln!("    [SoftMedium]: Shared: {}", *counter);
        });

        *cx.local.times += 1;
        hprintln!("    [SoftMedium]: Local: {}", *cx.local.times,);

        hprintln!("    [SoftMedium]: Finished");
    }

    /// Low priority SW task. It runs cooperatively with soft_low_2
    #[task(local = [times: u32 = 0], shared = [counter], priority = 1)]
    async fn soft_low_1(mut cx: soft_low_1::Context) {
        hprintln!("[SoftLow1]: Started");
        cx.shared.counter.lock(|counter| {
            *counter += 1;
            hprintln!("[SoftLow1]: Shared: {}", *counter);
        });
        // Yield to the other SW task
        yield_now("SoftLow1").await;

        // Safe access to local `static mut` variable
        *cx.local.times += 1;
        hprintln!("[SoftLow1]: Local: {}", *cx.local.times);

        hprintln!("[SoftLow1]: Finished");
    }

    /// Low priority SW task. It runs cooperatively with soft_low_2
    #[task(local = [times: u32 = 0], shared = [counter], priority = 1)]
    async fn soft_low_2(mut cx: soft_low_2::Context) {
        hprintln!("[SoftLow2]: Started");
        cx.shared.counter.lock(|counter| {
            *counter += 1;
            hprintln!("[SoftLow2]: Shared: {}", *counter);
        });

        // Yield to the other SW task
        yield_now("SoftLow2").await;

        // Safe access to local `static mut` variable
        *cx.local.times += 1;
        hprintln!("[SoftLow2]: Local: {}", *cx.local.times);

        hprintln!("[SoftLow2]: Finished");
    }

    /// High priority SW task
    #[task(local = [times: u32 = 0], shared = [counter], priority = 3)]
    async fn soft_high(mut cx: soft_high::Context) {
        hprintln!("        [SoftHigh]: Started");

        cx.shared.counter.lock(|counter| {
            *counter += 1;
            hprintln!("        [SoftHigh]: Shared: {}", counter);
        });

        // Safe access to local `static mut` variable
        *cx.local.times += 1;
        hprintln!("        [SoftHigh]: Local: {}", *cx.local.times);

        hprintln!("        [SoftHigh]: Finished");
    }
}
