#![no_std]

mod fake_temperature_sensor;
mod temperature_monitor;

// Actors
pub use fake_temperature_sensor::FakeTemperatureSensor;
pub use temperature_monitor::TemperatureMonitor;

// Messages
pub struct TemperatureAlert;

#[derive(Clone, Debug, PartialEq)]
pub struct TemperatureReadingCelsius(pub i32);
