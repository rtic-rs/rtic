use rtic_actor_traits::Post;

use crate::TemperatureReadingCelsius;

pub struct FakeTemperatureSensor<P>
where
    P: Post<TemperatureReadingCelsius>,
{
    delta: i32,
    outbox: P,
    temperature: i32,
}

// a real temperature sensor would use the embedded-hal traits (e.g. I2C) or some higher level trait
impl<P> FakeTemperatureSensor<P>
where
    P: Post<TemperatureReadingCelsius>,
{
    pub fn new(outbox: P, initial_temperature: i32, delta: i32) -> Self {
        Self {
            delta,
            outbox,
            temperature: initial_temperature,
        }
    }

    pub fn read(&mut self) {
        self.outbox
            .post(TemperatureReadingCelsius(self.temperature))
            .expect("OOM");
        self.temperature += self.delta;
    }
}

#[cfg(test)]
mod tests {
    use rtic_post_spy::PostSpy;

    use super::*;

    #[test]
    fn on_read_it_posts_reading() {
        let mut sensor = FakeTemperatureSensor::new(PostSpy::default(), 0, 0);
        sensor.read();

        let spy = sensor.outbox;
        let posted_messages = spy.posted_messages::<TemperatureReadingCelsius>();
        assert_eq!(1, posted_messages.count());
    }

    #[test]
    fn reading_starts_at_initial_temperature() {
        let initial_temperature = 1;
        let mut sensor = FakeTemperatureSensor::new(PostSpy::default(), initial_temperature, 0);
        sensor.read();

        let spy = sensor.outbox;
        let mut posted_messages = spy.posted_messages::<TemperatureReadingCelsius>();
        assert_eq!(
            Some(&TemperatureReadingCelsius(initial_temperature)),
            posted_messages.next()
        );
    }

    #[test]
    fn reading_changes_by_delta() {
        let initial_temperature = 42;
        let delta = 1;
        let mut sensor = FakeTemperatureSensor::new(PostSpy::default(), initial_temperature, delta);
        sensor.read();
        sensor.read();

        let spy = sensor.outbox;
        let mut posted_messages = spy.posted_messages::<TemperatureReadingCelsius>();
        assert_eq!(
            Some(&TemperatureReadingCelsius(initial_temperature)),
            posted_messages.next()
        );
        assert_eq!(
            Some(&TemperatureReadingCelsius(initial_temperature + delta)),
            posted_messages.next()
        );
    }
}
