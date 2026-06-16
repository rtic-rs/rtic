#![no_std]
#![no_main]

esp_bootloader_esp_idf::esp_app_desc!();

use esp_backtrace as _;

#[rtic::app(device = esp32, dispatchers = [FROM_CPU_INTR0])]
mod app {
    use esp_hal::uart::{Config, RxConfig, Uart, UartInterrupt};
    use esp_println::println;

    #[shared]
    struct Shared {
        byte_count: u32,
    }

    #[local]
    struct Local {
        uart: Uart<'static, esp_hal::Blocking>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let _ = esp_hal::init(esp_hal::Config::default());
        // TODO: really need to find a better/more ergonomic peripherals impl...
        let peripherals = cx.device;

        let config = Config::default().with_rx(
            RxConfig::default().with_fifo_full_threshold(1)
        );
        let mut uart = Uart::new(peripherals.UART0, config)
            .unwrap()
            .with_rx(peripherals.GPIO3);
        uart.listen(UartInterrupt::RxFifoFull | UartInterrupt::RxTimeout);

        println!("init");
        (Shared { byte_count: 0 }, Local { uart })
    }

    #[task(binds = UART0, local = [uart], shared = [byte_count], priority = 1)]
    fn uart0(mut cx: uart0::Context) {
        let uart = cx.local.uart;

        let mut buf = [0u8; 64];
        if let Ok(n) = uart.read_buffered(&mut buf) {
            if n > 0 {
                let s = core::str::from_utf8(&buf[..n]).unwrap_or("<non-utf8>");
                println!("rx: {:?}", s);
                cx.shared.byte_count.lock(|c| *c += n as u32);
            }
        }

        uart.clear_interrupts(UartInterrupt::RxFifoFull | UartInterrupt::RxTimeout);
    }

    #[idle(shared = [byte_count])]
    fn idle(mut cx: idle::Context) -> ! {
        let mut last = 0u32;
        loop {
            let current = cx.shared.byte_count.lock(|c| *c);
            if current != last {
                println!("total bytes received: {}", current);
                last = current;
            }
        }
    }
}
