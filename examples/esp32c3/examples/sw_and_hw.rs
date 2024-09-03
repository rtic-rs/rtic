#![no_main]
#![no_std]
#[rtic::app(device = esp32c3, dispatchers=[FROM_CPU_INTR0, FROM_CPU_INTR1])]
mod app {
    use esp_backtrace as _;
    use esp_hal::{
        gpio::{Event, GpioPin, Input, Io, Pull},
        peripherals::Peripherals,
    };
    use esp_println::println;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        button: Input<'static, GpioPin<9>>,
    }

    // do nothing in init
    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        println!("init");
        let peripherals = Peripherals::take();
        let io = Io::new_no_bind_interrupt(peripherals.GPIO, peripherals.IO_MUX);
        let mut button = Input::new(io.pins.gpio9, Pull::Up);
        button.listen(Event::FallingEdge);
        foo::spawn().unwrap();
        (Shared {}, Local { button })
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        println!("idle");
        loop {}
    }

    #[task(priority = 5)]
    async fn foo(_: foo::Context) {
        bar::spawn().unwrap(); //enqueue low prio task
        println!("Inside high prio task, press button now!");
        let mut x = 0;
        while x < 5000000 {
            x += 1; //burn cycles
            esp_hal::riscv::asm::nop();
        }
        println!("Leaving high prio task.");
    }

    #[task(priority = 2)]
    async fn bar(_: bar::Context) {
        println!("Inside low prio task, press button now!");
        let mut x = 0;
        while x < 5000000 {
            x += 1; //burn cycles
            esp_hal::riscv::asm::nop();
        }
        println!("Leaving low prio task.");
    }

    #[task(binds=GPIO, local=[button], priority = 3)]
    fn gpio_handler(cx: gpio_handler::Context) {
        cx.local.button.clear_interrupt();
        println!("button");
    }
}
