#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_monotonics::systick_monotonic::*;

    rtic_monotonics::make_systick_timer_queue!(TIMER);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init");

        let systick = Systick::start(cx.core.SYST, 12_000_000);
        TIMER.initialize(systick);

        foo::spawn().ok();
        bar::spawn().ok();
        baz::spawn().ok();

        (Shared {}, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // debug::exit(debug::EXIT_SUCCESS);
        loop {
            // hprintln!("idle");
            cortex_m::asm::wfi(); // put the MCU in sleep mode until interrupt occurs
        }
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        hprintln!("hello from foo");
        TIMER.delay(100.millis()).await;
        hprintln!("bye from foo");
    }

    #[task]
    async fn bar(_cx: bar::Context) {
        hprintln!("hello from bar");
        TIMER.delay(200.millis()).await;
        hprintln!("bye from bar");
    }

    #[task]
    async fn baz(_cx: baz::Context) {
        hprintln!("hello from baz");
        TIMER.delay(300.millis()).await;
        hprintln!("bye from baz");

        debug::exit(debug::EXIT_SUCCESS);
    }
}
