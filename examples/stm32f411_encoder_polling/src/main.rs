#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {

    use stm32f4xx_hal::i2c::I2c;
    use stm32f4xx_hal::i2c::Mode;
    use stm32f4xx_hal::{
        gpio::{self, Input, Output, PushPull},
        pac::{Peripherals, I2C1, TIM1, TIM2},
        prelude::*,
        timer,
        timer::{CounterHz, Event, Timer2},
    };

    use rotary_encoder_embedded::standard::StandardMode;
    use rotary_encoder_embedded::{Direction, RotaryEncoder};

    use hd44780_driver::bus::I2CBus;
    use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};

    use defmt_rtt as _;

    pub struct Knob {
        rotary_encoder: RotaryEncoder<StandardMode, gpio::PB12<Input>, gpio::PB13<Input>>,
        value: u8,
    }

    impl Knob {
        pub fn new(
            rotary_encoder: RotaryEncoder<StandardMode, gpio::PB12<Input>, gpio::PB13<Input>>,
        ) -> Knob {
            Knob {
                rotary_encoder: rotary_encoder,
                value: 0_u8,
            }
        }
    }

    // Set the I2C address of the PCF8574 located on the back of the HD44780.
    // Check the jumpers A0, A1 and A2 and the datasheet.
    const LCD_I2C_ADDRESS: u8 = 0x27;

    // Resources shared between tasks
    #[shared]
    struct Shared {
        delay: timer::DelayMs<TIM1>,
        lcd: HD44780<I2CBus<I2c<I2C1>>>,
    }

    // Local resources to specific tasks (cannot be shared)
    #[local]
    struct Local {
        led: gpio::PC13<Output<PushPull>>,
        tim2_timer: CounterHz<TIM2>,
        knob_1: Knob,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let dp: Peripherals = ctx.device;

        // Configure and obtain handle for delay abstraction using TIM1
        // Promote RCC structure to HAL to be able to configure clocks
        let rcc = dp.RCC.constrain();

        // Configure the system clocks 25 MHz must be used for HSE
        // on the Blackpill-STM32F411CE board according to manual
        let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();

        let mut delay = dp.TIM1.delay_ms(&clocks);

        // Configure the LED pin as a push pull output and obtain handle
        // On the Blackpill STM32F411CEU6 there is an on-board LED connected to pin PC13
        // Promote the GPIOC PAC struct
        let gpioc = dp.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output();

        // Configure the button pin as input and obtain handle
        // On the Blackpill STM32F411CEU6 there is a button connected to pin PA0
        // Promote the GPIOB PAC struct
        let gpiob = dp.GPIOB.split();

        // Configure Pins connected to encoder as floating input (only if your encoder
        // board already has pull-up resistors, use 'into_pull_up_input' otherwise)
        // and Obtain Handle.
        let enc_1_a = gpiob.pb12.into_floating_input();
        let enc_1_b = gpiob.pb13.into_floating_input();

        // Instantiate RotaryEncoder struct
        let encoder_1 = RotaryEncoder::new(enc_1_a, enc_1_b).into_standard_mode();

        // Instantiate Knob struct to hold 'encoder_1' and its current value
        let knob_1 = Knob::new(encoder_1);

        // Instantiate TIM2 that will be used to poll the encoders
        let tim2_timer = Timer2::new(dp.TIM2, &clocks);
        let mut tim2_timer = tim2_timer.counter_hz();

        // Get SDA and SCL pins for I2C1
        let sda = gpiob.pb7;
        let scl = gpiob.pb6;

        // Instantiate I2C1 bus that will be used to communicate with the
        // LCD display.
        let i2c = I2c::new(
            dp.I2C1,
            (scl, sda),
            Mode::Standard {
                frequency: 400.kHz(),
            },
            &clocks,
        );

        let mut lcd = HD44780::new_i2c(i2c, LCD_I2C_ADDRESS, &mut delay).expect("Init LCD failed");

        let _ = lcd.reset(&mut delay);
        let _ = lcd.clear(&mut delay);
        let _ = lcd.set_display_mode(
            DisplayMode {
                display: Display::On,
                cursor_visibility: Cursor::Invisible,
                cursor_blink: CursorBlink::Off,
            },
            &mut delay,
        );
        let _ = lcd.set_cursor_pos(57, &mut delay);
        let _ = lcd.write_str("RTIC + I2C + Encoder", &mut delay);
        delay.delay_ms(1500);

        let _ = lcd.clear(&mut delay);
        let _ = lcd.set_cursor_pos(68, &mut delay);
        let _ = lcd.write_str("Encoder 1", &mut delay);

        // Start the timer at 2 kHz
        tim2_timer.start(2000.Hz()).unwrap();

        // Generate an interrupt when the timer expires
        tim2_timer.listen(Event::Update);

        (
            // Initialization of shared resources.
            Shared { delay, lcd },
            // Initialization of task local resources
            Local {
                led,
                tim2_timer,
                knob_1,
            },
        )
    }

    // Background task, runs whenever no other tasks are running
    #[idle(shared = [])]
    fn idle(mut _ctx: idle::Context) -> ! {
        loop {}
    }

    // Handle the IRQ generated when the TIM2 times out
    #[task(binds = TIM2, local=[led, tim2_timer, knob_1], shared=[lcd, delay])]
    fn tim2_timeout_interrupt_handler(mut ctx: tim2_timeout_interrupt_handler::Context) {
        ctx.local.tim2_timer.clear_all_flags();
        let led = &mut ctx.local.led;
        led.toggle();

        //Obtain 2 shared resources: lcd and delay.
        let delay = &mut ctx.shared.delay;
        let lcd = &mut ctx.shared.lcd;

        // Update the encoder, which will compute and return its direction
        match ctx.local.knob_1.rotary_encoder.update() {
            Direction::Clockwise => {
                if ctx.local.knob_1.value < 255 {
                    ctx.local.knob_1.value += 1;
                    defmt::info!("Going UP! {:?}", ctx.local.knob_1.value);
                    let current_value = ctx.local.knob_1.value;
                    (delay, lcd).lock(|delay, lcd| {
                        //Move cursor to the middle of the third line and update value
                        let _ = lcd.set_cursor_pos(27, delay);
                        let _ = lcd.write_bytes(&u8_to_str(current_value), delay);
                    })
                }
            }
            Direction::Anticlockwise => {
                if ctx.local.knob_1.value > 0 {
                    ctx.local.knob_1.value -= 1;
                    defmt::info!("Going DN! {:?}", ctx.local.knob_1.value);
                    let current_value = ctx.local.knob_1.value;
                    (delay, lcd).lock(|delay, lcd| {
                        //Move cursor to the middle of the third line and update value
                        let _ = lcd.set_cursor_pos(27, delay);
                        let _ = lcd.write_bytes(&u8_to_str(current_value), delay);
                    })
                }
            }
            Direction::None => {
                // Do nothing
            }
        }
    }

    // This auxiliary function is in charge of converting a u8 number into
    // a 3-char string representation without doing memory allocation.
    // 0  is represented as "000"
    // 84 is represented as "084"
    fn u8_to_str(n: u8) -> [u8; 3] {
        let mut buffer = [b'0'; 3]; // Initialize the buffer with '0'
        let mut num = n;
        let mut i = 2;

        // Fill the buffer with digits from the end to the start
        loop {
            buffer[i] = (num % 10) + b'0';
            num /= 10;
            if num == 0 {
                break;
            }
            i -= 1;
        }

        buffer
    }
}
