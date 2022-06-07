#![no_main]
#![no_std]

use firmware as _;

#[rtic::app(device = nrf52840_hal::pac, dispatchers = [RADIO])]
mod app {
    use actors::{
        DoTemperatureRead, FakeTemperatureSensor, TemperatureAlert, TemperatureMonitor,
        TemperatureReadingCelsius,
    };
    use rtic_actor_traits::Receive;
    use systick_monotonic::*;

    // configuration
    const TEMPERATURE_THRESHOLD: i32 = 37;
    const INITIAL_FAKE_TEMPERATURE: i32 = 35;
    const FAKE_TEMPERATURE_DELTA: i32 = 1;

    // app-specific actors
    struct AlertHandler;

    impl Receive<TemperatureAlert> for AlertHandler {
        fn receive(&mut self, _: TemperatureAlert) {
            defmt::error!("temperature alert");
            firmware::exit()
        }
    }

    struct TemperatureTracer;

    impl Receive<TemperatureReadingCelsius> for TemperatureTracer {
        fn receive(&mut self, reading: TemperatureReadingCelsius) {
            defmt::println!("temperature: {} C", reading.0);
        }
    }

    #[actors]
    struct Actors {
        #[subscribe(DoTemperatureRead)]
        temperature_sensor: FakeTemperatureSensor<Poster>,

        #[init(AlertHandler)]
        #[subscribe(TemperatureAlert)]
        alert_handler: AlertHandler,

        #[subscribe(TemperatureReadingCelsius)] // <- broadcast
        temperature_monitor: TemperatureMonitor<Poster>,

        #[init(TemperatureTracer)]
        #[subscribe(TemperatureReadingCelsius)] // <- broadcast
        temperature_tracer: TemperatureTracer,
    }

    #[local]
    struct Local {
        poster: Poster,
    }

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<100>; // 100 Hz

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 48_000_000);

        let poster = cx.poster;
        let temperature_monitor = TemperatureMonitor::new(poster, TEMPERATURE_THRESHOLD);
        let temperature_sensor =
            FakeTemperatureSensor::new(poster, INITIAL_FAKE_TEMPERATURE, FAKE_TEMPERATURE_DELTA);

        // kick start the system
        periodic::spawn().expect("OOM");

        (
            Shared {},
            Local { poster },
            init::Monotonics(mono),
            Actors {
                temperature_monitor,
                temperature_sensor,
            },
        )
    }

    #[task(local = [poster])]
    fn periodic(cx: periodic::Context) {
        // input to the actor network
        cx.local.poster.post(DoTemperatureRead).expect("OOM");

        periodic::spawn_after(1.secs()).expect("OOM");
    }

    #[shared]
    struct Shared {}
}
