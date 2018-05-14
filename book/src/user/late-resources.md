# Runtime initialized resources

Normal `static` variables in Rust must be assigned an initial value when declared, i.e. at compile
time. Resources don't have this limitation and can be initialized at *runtime*; these resources are
called "late resources" because they are initialized *late*. The initial values of late resources
must be returned by the `init` function.

Consider the following example where we load a cryptographic key from EEPROM and then use the key in
a task.

``` rust
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm;

use cortex_m_rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static KEY: [u8; 256];
    },

    tasks: {
        exti0: {
            interrupt: USART1, // data arrived via the serial interface
            resources: [KEY],
        },
    },
}

fn init(ctxt: init::Context) -> init::LateResources {
    let key = load_from_eeprom();

    init::LateResources {
        KEY: key,
    }
}

// omitted: `idle`

fn usart1(ctxt: usart1::Context) {
    let key: &[u8; 256] = ctxt.resources.KEY;

    // use key to decrypt incoming data
}
```
