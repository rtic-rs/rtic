#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {

    use stm32f4xx_hal::{
        gpio::{self, Edge, Input, Output, PushPull},
        pac::{TIM1, TIM2},
        prelude::*,
        timer::{self, Channel},
    };

    use defmt_rtt as _;

    #[shared]
    struct Shared {
        delayval: u32,
        duty_percent: u8,
        increasing: bool,
    }

    #[local]
    struct Local {
        button: gpio::PA0<Input>,
        led: gpio::PC13<Output<PushPull>>,
        delay: timer::DelayMs<TIM2>,

        pwm: timer::PwmHz<TIM1, timer::ChannelBuilder<TIM1, 1>>,

        dwt: cortex_m::peripheral::DWT,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let mut dp = ctx.device;
        let mut cp = ctx.core;

        defmt::info!("BOOT: init start");

        let rcc = dp.RCC.constrain();

        let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();

        // =========================
        // DWT INIT
        // =========================

        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();

        let dwt = cp.DWT;

        defmt::info!("DWT ready");

        let delay = dp.TIM2.delay_ms(&clocks);

        let gpioc = dp.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output();

        let gpioa = dp.GPIOA.split();

        // =========================
        // PWM TIM1 CH2 on PA9
        // =========================

        let pwm_pin = gpioa.pa9.into_alternate::<1>();

        let mut pwm = dp
            .TIM1
            .pwm_hz(timer::Channel2::new(pwm_pin), 1.kHz(), &clocks);

        let max = pwm.get_max_duty();

        pwm.set_duty(Channel::C2, max / 2);
        pwm.enable(Channel::C2);

        defmt::info!("PWM ready");

        let mut button = gpioa.pa0.into_pull_up_input();

        let mut syscfg = dp.SYSCFG.constrain();

        button.make_interrupt_source(&mut syscfg);
        button.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        button.enable_interrupt(&mut dp.EXTI);

        defmt::info!("IRQ ready");

        (
            Shared {
                delayval: 2000,
                duty_percent: 50,
                increasing: true,
            },
            Local {
                button,
                led,
                delay,
                pwm,
                dwt,
            },
        )
    }

    #[idle(local = [led, delay], shared = [delayval])]
    fn idle(mut ctx: idle::Context) -> ! {
        defmt::info!("IDLE running");

        loop {
            ctx.local.led.set_high();

            let d = ctx.shared.delayval.lock(|d| *d);
            ctx.local.delay.delay_ms(d);

            ctx.local.led.set_low();

            let d = ctx.shared.delayval.lock(|d| *d);
            ctx.local.delay.delay_ms(d);
        }
    }

    #[task(
        binds = EXTI0,
        local = [button, pwm, dwt],
        shared = [delayval, duty_percent, increasing]
    )]
    fn gpio_interrupt_handler(mut ctx: gpio_interrupt_handler::Context) {
        defmt::info!("IRQ");

        ctx.shared.delayval.lock(|del| {
            *del -= 100;
            if *del < 200 {
                *del = 2000;
            }
        });

        let duty = (&mut ctx.shared.duty_percent, &mut ctx.shared.increasing).lock(|duty, inc| {
            if *inc {
                if *duty >= 100 {
                    *inc = false;
                    *duty -= 5;
                } else {
                    *duty += 5;
                }
            } else {
                if *duty <= 5 {
                    *inc = true;
                    *duty += 5;
                } else {
                    *duty -= 5;
                }
            }

            *duty
        });

        let max = ctx.local.pwm.get_max_duty();
        let duty_ticks = (max as u32 * duty as u32 / 100) as u16;

        // Start cycle count
        let start = ctx.local.dwt.cyccnt.read();

        ctx.local.pwm.set_duty(Channel::C2, duty_ticks);

        // Read cycle count after updating duty cycle
        let cycles = ctx.local.dwt.cyccnt.read().wrapping_sub(start);

        // CPU frequency in MHz (STM32F411 = 84 MHz = 84 cycles per microsecond)
        let cpu_mhz = 84u32;

        // convert cycles → microseconds
        let micros = cycles / cpu_mhz;

        defmt::info!(
            "PWM duty={}%, update={} cycles ({} us)",
            duty,
            cycles,
            micros
        );

        ctx.local.button.clear_interrupt_pending_bit();
    }
}
