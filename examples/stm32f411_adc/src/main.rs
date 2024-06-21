#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {

    use stm32f4xx_hal::pac::ADC1;
    use stm32f4xx_hal::{
        adc::{
            config::{AdcConfig, SampleTime},
            Adc,
        },
        gpio::{self, Analog, Edge, Input, Output, PushPull},
        pac::TIM1,
        prelude::*,
        timer,
    };

    use defmt_rtt as _;

    // A simple placeholder for the analog pin
    struct Potentiometer {
        analog_input: gpio::PA1<Analog>,
    }

    // Resources shared between tasks
    #[shared]
    struct Shared {
        delayval: u32,
        adc_module: Adc<ADC1>,
    }

    // Local resources to specific tasks (cannot be shared)
    #[local]
    struct Local {
        button: gpio::PA0<Input>,
        pot_instance: Potentiometer,
        led: gpio::PC13<Output<PushPull>>,
        delay: timer::DelayMs<TIM1>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let mut dp = ctx.device;

        // Configure and obtain handle for delay abstraction
        // 1) Promote RCC structure to HAL to be able to configure clocks
        let rcc = dp.RCC.constrain();

        // 2) Configure the system clocks
        // 25 MHz must be used for HSE on the Blackpill-STM32F411CE board according to manual
        let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();

        // 3) Create delay handle
        let delay = dp.TIM1.delay_ms(&clocks);

        // Configure the LED pin as a push pull ouput and obtain handle
        // On the Blackpill STM32F411CEU6 there is an on-board LED connected to pin PC13
        // 1) Promote the GPIOC PAC struct
        let gpioc = dp.GPIOC.split();

        // 2) Configure PORTC OUTPUT Pins and Obtain Handle
        let led = gpioc.pc13.into_push_pull_output();

        // Configure the button pin as input and obtain handle
        // On the Blackpill STM32F411CEU6 there is a button connected to pin PA0
        // 3) Promote the GPIOA PAC struct
        let gpioa: gpio::gpioa::Parts = dp.GPIOA.split();
        // 4) Configure Pin and Obtain Handle
        let mut button = gpioa.pa0.into_pull_up_input();

        // 5) Configure pin A1 of the blackpill to be of type analog
        // the input does not need to be mutable since we are only reading it.
        let analog_input = gpioa.pa1.into_analog();

        // 6) Configure the ADC modulke for single-shot conversion
        let mut adc = Adc::adc1(dp.ADC1, true, AdcConfig::default());
        // Calibrate by calculates the system VDDA by sampling the internal VREF
        // channel and comparing the result with the value stored at the factory.
        adc.calibrate();

        let pot_instance = Potentiometer {
            analog_input: analog_input,
        };

        // Configure Button Pin for Interrupts
        // 7) Promote SYSCFG structure to HAL to be able to configure interrupts
        let mut syscfg = dp.SYSCFG.constrain();
        // 8) Make button an interrupt source
        button.make_interrupt_source(&mut syscfg);
        // 9) Configure the interruption to be triggered on a rising edge
        button.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        // 10) Enable gpio interrupt for button
        button.enable_interrupt(&mut dp.EXTI);

        (
            // Initialization of shared resources. In this case delay value and the ADC instance
            Shared {
                delayval: 2000_u32,
                adc_module: adc,
            },
            // Initialization of task local resources
            Local {
                button,
                pot_instance,
                led,
                delay,
            },
        )
    }

    // Background task, runs whenever no other tasks are running
    #[idle(local = [led, delay], shared = [delayval])]
    fn idle(mut ctx: idle::Context) -> ! {
        let led = ctx.local.led;
        let delay = ctx.local.delay;
        loop {
            // Turn On LED
            led.set_high();
            // Obtain shared delay variable and delay
            delay.delay_ms(ctx.shared.delayval.lock(|del| *del));
            // Turn off LED
            led.set_low();
            // Obtain shared delay variable and delay
            delay.delay_ms(ctx.shared.delayval.lock(|del| *del));
        }
    }

    // Handle the IRQ generated when the button is pressed and interact with local and shared resources.
    #[task(binds = EXTI0, local = [button, pot_instance], shared=[delayval, adc_module])]
    fn gpio_interrupt_handler(mut ctx: gpio_interrupt_handler::Context) {
        ctx.shared.delayval.lock(|del| {
            *del = *del - 100_u32;
            if *del < 200_u32 {
                *del = 2000_u32;
            }
            *del
        });

        ctx.shared.delayval.lock(|del| {
            defmt::info!("Current delay value {:?}", del);
        });

        // Obtain the Potentiometer instance that belongs to this task ONLY
        let analog_input = &ctx.local.pot_instance.analog_input;

        // Obtain the shared instance of Adc and do one conversion of the value seen
        ctx.shared.adc_module.lock(|adc_module| {
            let sample = adc_module.convert(analog_input, SampleTime::Cycles_480);
            defmt::info!("Current ADC value {:?}\n", sample);
        });

        ctx.local.button.clear_interrupt_pending_bit();
    }
}
