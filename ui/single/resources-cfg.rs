#![no_main]

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[cfg(never)]
        #[init(0)]
        o1: u32, // init

        #[cfg(never)]
        #[init(0)]
        o2: u32, // idle

        #[cfg(never)]
        #[init(0)]
        o3: u32, // EXTI0

        #[cfg(never)]
        #[init(0)]
        o4: u32, // idle

        #[cfg(never)]
        #[init(0)]
        o5: u32, // EXTI1

        #[cfg(never)]
        #[init(0)]
        o6: u32, // init

        #[cfg(never)]
        #[init(0)]
        s1: u32, // idle & EXTI0

        #[cfg(never)]
        #[init(0)]
        s2: u32, // EXTI0 & EXTI1

        #[cfg(never)]
        #[init(0)]
        s3: u32,
    }

    #[init(resources = [o1, o4, o5, o6, s3])]
    fn init(c: init::Context) {
        c.resources.o1;
        c.resources.o4;
        c.resources.o5;
        c.resources.o6;
        c.resources.s3;
    }

    #[idle(resources = [o2, &o4, s1, &s3])]
    fn idle(c: idle::Context) -> ! {
        c.resources.o2;
        c.resources.o4;
        c.resources.s1;
        c.resources.s3;

        loop {}
    }

    #[task(binds = UART0, resources = [o3, s1, s2, &s3])]
    fn uart0(c: uart0::Context) {
        c.resources.o3;
        c.resources.s1;
        c.resources.s2;
        c.resources.s3;
    }

    #[task(binds = UART1, resources = [s2, &o5])]
    fn uart1(c: uart1::Context) {
        c.resources.s2;
        c.resources.o5;
    }
};
