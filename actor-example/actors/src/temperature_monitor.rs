use rtic_actor_traits::{Post, Receive};

use crate::{TemperatureAlert, TemperatureReadingCelsius};

pub struct TemperatureMonitor<P>
where
    P: Post<TemperatureAlert>,
{
    outbox: P,
    threshold: i32,
}

impl<P> TemperatureMonitor<P>
where
    P: Post<TemperatureAlert>,
{
    pub fn new(outbox: P, threshold: i32) -> Self {
        Self { outbox, threshold }
    }
}

impl<P> Receive<TemperatureReadingCelsius> for TemperatureMonitor<P>
where
    P: Post<TemperatureAlert>,
{
    fn receive(&mut self, temperature: TemperatureReadingCelsius) {
        if temperature.0 >= self.threshold {
            self.outbox.post(TemperatureAlert).ok().expect("OOM");
        }
    }
}

#[cfg(test)]
mod tests {
    use rtic_post_spy::PostSpy;

    use super::*;

    #[test]
    fn when_temperature_is_above_threshold_it_posts_alert_once() {
        let mut monitor = TemperatureMonitor::new(PostSpy::default(), 0);

        // manually send a message
        let message = TemperatureReadingCelsius(1);
        monitor.receive(message);

        let spy = monitor.outbox;
        let posted_messages = spy.posted_messages::<TemperatureAlert>();
        assert_eq!(1, posted_messages.count());
    }

    #[test]
    fn when_temperature_is_below_threshold_it_does_not_post_alert() {
        let mut monitor = TemperatureMonitor::new(PostSpy::default(), 0);

        let message = TemperatureReadingCelsius(-1);
        monitor.receive(message);

        let spy = monitor.outbox;
        let posted_messages = spy.posted_messages::<TemperatureAlert>();
        assert_eq!(0, posted_messages.count());
    }
}
