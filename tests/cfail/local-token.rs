#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
        },
    }
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}

task!(EXTI0, exti0, Old {
    token: Option<Threshold> = None;
});

fn exti0(nt: &mut Threshold, old: &mut Old, _r: EXTI0::Resources) {
    if let Some(ot) = old.token.take() {
        let _: (Threshold, Threshold) = (*nt, ot);
        //~^ error cannot move out of borrowed content

        return
    }

    // ERROR can't store a threshold token in a local variable, otherwise you
    // would end up with two threshold tokens in a task (see `if let` above)
    old.token = Some(*nt);
    //~^ error cannot move out of borrowed content
}
