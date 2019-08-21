# Interrupt configuration

Interrupts are core to the operation of RTFM applications. Correctly setting
interrupt priorities and ensuring they remain fixed at runtime is a requisite
for the memory safety of the application.

The RTFM framework exposes interrupt priorities as something that is declared at
compile time. However, this static configuration must be programmed into the
relevant registers during the initialization of the application. The interrupt
configuration is done before the `init` function runs.

This example gives you an idea of the code that the RTFM framework runs:

``` rust
#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(c: init::Context) {
        // .. user code ..
    }

    #[idle]
    fn idle(c: idle::Context) -> ! {
        // .. user code ..
    }

    #[interrupt(binds = UART0, priority = 2)]
    fn foo(c: foo::Context) {
        // .. user code ..
    }
};
```

The framework generates an entry point that looks like this:

``` rust
// the real entry point of the program
#[no_mangle]
unsafe fn main() -> ! {
    // transforms a logical priority into a hardware / NVIC priority
    fn logical2hw(priority: u8) -> u8 {
        use lm3s6965::NVIC_PRIO_BITS;

        // the NVIC encodes priority in the higher bits of a bit
        // also a bigger numbers means lower priority
        ((1 << NVIC_PRIORITY_BITS) - priority) << (8 - NVIC_PRIO_BITS)
    }

    cortex_m::interrupt::disable();

    let mut core = cortex_m::Peripheral::steal();

    core.NVIC.enable(Interrupt::UART0);

    // value specified by the user
    let uart0_prio = 2;

    // check at compile time that the specified priority is within the supported range
    let _ = [(); (1 << NVIC_PRIORITY_BITS) - (uart0_prio as usize)];

    core.NVIC.set_priority(Interrupt::UART0, logical2hw(uart0_prio));

    // call into user code
    init(/* .. */);

    // ..

    cortex_m::interrupt::enable();

    // call into user code
    idle(/* .. */)
}
```
