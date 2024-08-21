#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true,  dispatchers = [SPI1, SPI2])]
mod app {

    use defmt_rtt as _;
    use rtic_monotonics::systick::prelude::*;
    use rtic_sync::channel::{Receiver, Sender};
    use rtic_sync::make_channel;
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

    systick_monotonic!(Mono, 100);

    // A simple placeholder for the analog pin
    struct Potentiometer {
        analog_input: gpio::PA1<Analog>,
    }

    // An enum specifying the type of messages the printer actor running on
    // a software expects
    enum Message {
        PotentiometerValue(u16),
        Ping,
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
        sender_from_exti0: Sender<'static, Message, 8>,
        button: gpio::PA0<Input>,
        pot_instance: Potentiometer,
        led: gpio::PC13<Output<PushPull>>,
        delay: timer::DelayMs<TIM1>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        Mono::start(ctx.core.SYST, 12_000_000);
        let mut dp = ctx.device;

        // Configure and obtain handle for delay abstraction
        // 1) Promote RCC structure to HAL to be able to configure clocks
        let rcc = dp.RCC.constrain();

        // 2) Configure the system clocks
        // 25 MHz must be used for HSE on the Blackpill-STM32F411CE board according to manual
        let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();

        // 3) Create delay handle
        let delay = dp.TIM1.delay_ms(&clocks);

        // Configure the LED pin as a push pull output and obtain handle
        // On the Blackpill STM32F411CEU6 there is an on-board LED connected to pin PC13
        // 4) Promote the GPIOC PAC struct
        let gpioc = dp.GPIOC.split();

        // 5) Configure PORTC OUTPUT Pins and Obtain Handle
        let led = gpioc.pc13.into_push_pull_output();

        // Configure the button pin as input and obtain handle
        // On the Blackpill STM32F411CEU6 there is a button connected to pin PA0
        // 6) Promote the GPIOA PAC struct
        let gpioa: gpio::gpioa::Parts = dp.GPIOA.split();
        // 7) Configure Pin and Obtain Handle
        let mut button = gpioa.pa0.into_pull_up_input();

        // 8) Configure pin A1 of the blackpill to be of type analog
        // the input does not need to be mutable since we are only reading it.
        let analog_input = gpioa.pa1.into_analog();

        // 9) Configure the ADC modulke for single-shot conversion
        let mut adc = Adc::adc1(dp.ADC1, true, AdcConfig::default());
        // Calibrate by calculates the system VDDA by sampling the internal VREF
        // channel and comparing the result with the value stored at the factory.
        adc.calibrate();

        let pot_instance = Potentiometer {
            analog_input: analog_input,
        };

        // Configure Button Pin for Interrupts
        // 10) Promote SYSCFG structure to HAL to be able to configure interrupts
        let mut syscfg = dp.SYSCFG.constrain();
        // 11) Make button an interrupt source
        button.make_interrupt_source(&mut syscfg);
        // 12) Configure the interruption to be triggered on a rising edge
        button.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        // 13) Enable gpio interrupt for button
        button.enable_interrupt(&mut dp.EXTI);

        // 14) Create a channel
        let (tx_to_printer, rx) = make_channel!(Message, 8);

        // 15) Spawn printer_actor and start listerning
        printer_actor::spawn(rx).unwrap();

        let sender_from_exti0 = tx_to_printer.clone();
        let sender_from_pinger = tx_to_printer.clone();
        pinger::spawn(sender_from_pinger).unwrap();

        (
            // Initialization of shared resources. In this case delay value and the ADC instance
            Shared {
                delayval: 2000_u32,
                adc_module: adc,
            },
            // Initialization of task local resources
            Local {
                sender_from_exti0,
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
    #[task(binds = EXTI0, local = [sender_from_exti0, button, pot_instance], shared=[delayval, adc_module])]
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

        let send_to_printer = ctx.local.sender_from_exti0;

        // Obtain the shared instance of Adc and do one conversion of the value seen
        ctx.shared.adc_module.lock(|adc_module| {
            let sample = adc_module.convert(analog_input, SampleTime::Cycles_480);

            // Now that we have the sampled value, we want to pass it to the software
            // task. Since we are in a non-async context, we must use 'try_send' method
            // and process the result
            let send_result = send_to_printer.try_send(Message::PotentiometerValue(sample));
            match send_result {
                Ok(_) => {}
                Err(_error) => {
                    defmt::error!("EXTI0 handler could not send message to printer actor");
                }
            }
        });

        ctx.local.button.clear_interrupt_pending_bit();
    }

    // The printer actor is a software task that listens to an MPSC channel and prints
    // the incoming Message from other tasks (Hardware or Software)
    #[task(priority = 1)]
    async fn printer_actor(_: printer_actor::Context, mut rx: Receiver<'_, Message, 8>) {
        loop {
            let maybe_new_message = rx.recv().await;
            match maybe_new_message {
                Ok(message) => match message {
                    Message::PotentiometerValue(value) => {
                        defmt::info!(
                            "Printer actor received a new PotentiometerValue: {:?} from hardware task.\n",
                            value
                        );
                    }
                    Message::Ping => {
                        defmt::info!("Printer actor received a PING from software task");
                    }
                },
                Err(error) => {
                    panic!("Receiver error {:?}", error);
                }
            }
        }
    }

    // This software task sends a Message of type Ping to printer_actor every 25000 millis
    #[task(priority = 1)]
    async fn pinger(_: pinger::Context, mut sender_from_pinger: Sender<'_, Message, 8>) {
        loop {
            let _ = sender_from_pinger.send(Message::Ping).await;
            Mono::delay(25000.millis()).await;
        }
    }
}
