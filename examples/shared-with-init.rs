//! `examples/shared-with-init.rs`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use panic_halt as _;
use rtfm::app;

pub struct MustBeSend;

#[app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(None)]
        shared: Option<MustBeSend>,
    }

    #[init(resources = [shared])]
    fn init(c: init::Context) {
        // this `message` will be sent to task `UART0`
        let message = MustBeSend;
        *c.resources.shared = Some(message);

        rtfm::pend(Interrupt::UART0);
    }

    #[task(binds = UART0, resources = [shared])]
    fn uart0(c: uart0::Context) {
        if let Some(message) = c.resources.shared.take() {
            // `message` has been received
            drop(message);

            debug::exit(debug::EXIT_SUCCESS);
        }
    }
};
