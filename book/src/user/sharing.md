# Resource sharing

We mentioned that in RTFM message passing is preferred over sharing state but sometimes the need of
shared state arises so let's look at an example.

Let's say we have an application with three tasks: one reads data from an accelerometer, the other
reads data from a gyroscope and the last one processes both the accelerometer and gyroscope data.
The first two tasks run periodically at 1 KHz (one thousand times per second); the third task must
start after the other two tasks are done and consumes the data each task produces. Here's one way to
implement such program:

``` rust
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm;

use cortex_m_rtfm::app;

struct Acceleration { x: u16, y: u16, z: u16 }

struct AngularRate { x: u16, y: u16, z: u16 }

enum Data {
    Empty,
    Acceleration(Acceleration),
    AngularRate(AngularRate),
}

app! {
    device: stm32f103xx,

    resources: {
        static DATA: Data = Data::Empty;

        // omitted: other resources
    },

    tasks: {
        accelerometer: {
            resources: [ACCELEROMETER, DATA],

            schedule_now: [process],

            // priority: 1,

            // omitted: interrupt source
        },

        gyroscope: {
            resources: [GYROSCOPE, DATA],

            schedule_now: [process],

            // priority: 1,

            // omitted: interrupt source
        },

        process: {
            input: (Acceleration, AngularRate),
        },
    }
}

// omitted: `init`, `idle` and `process`

fn accelerometer(ctxt: accelerometer::Context) {
    let accelerometer = ctxt.resources.ACCELEROMETER;
    let acceleration = accelerometer.read();

    let t = &mut ctxt.threshold;

    let angular_rate = {
        let data: &mut Data = ctxt.resources.DATA.borrow_mut(t);

        match *data {
            // store data
            Data::Empty => {
                *data = Data::Acceleration(acceleration);
                None
            },

            // overwrite old data
            Data::Acceleration(..) => {
                *data = Data::Acceleration(acceleration);
                None
            },

            // data pair is ready
            Data::AngularRate(angular_rate) => {
                *data = Data::Empty;
                Some(angular_rate)
            },
        }
    };

    if let Some(angular_rate) = angular_rate {
        ctxt.tasks.process.schedule_now(t, (acceleration, angular_rate)).unwrap();
    }
}

fn gyroscope(ctxt: accelerometer::Context) {
    let gyroscope = ctxt.resources.GYROSCOPE;
    let angular_rate = gyroscope.read();

    let t = &mut ctxt.threshold;

    let acceleration = {
        let data = ctxt.resources.DATA.borrow_mut(t);

        match *data {
            // store data
            Data::Empty => {
                *data = Data::AngularRate(angular_rate);
                None
            },

            // data pair is ready
            Data::Acceleration(acceleration) => {
                *data = Data::Empty;
                Some(acceleration)
            },

            // overwrite old data
            Data::AngularRate(angular_rate) => {
                *data = Data::AngularRate(angular_rate);
                None
            },
        }
    };

    if let Some(acceleration) = acceleration {
        ctxt.tasks.process.schedule_now(t, (acceleration, angular_rate)).unwrap();
    }
}
```

In this program the tasks `acceloremeter` and `gyroscope` share the `DATA` resource. This resource
can contain either sensor reading or no data at all. The idea is that either sensor task can start
the `process` task but only the one that has both readings will do. That's where `DATA` comes in: if
the `accelerometer` task happens first it stores its reading into `DATA`; then when the `gyroscope`
task occurs it *takes* the acceloremeter reading from `DATA`, leaving it empty, and schedules the
`process` task passing both readings. This setup also supports the other scenario where the
`gyroscope` task starts before the `accelerometer` task.

In this particular case both sensor tasks operate at the same priority so preemption is not
possible: if both tasks need to run at about the same time one will run *after* the other. Without
preemption a data race is not possible so each task can directly borrow (`borrow` / `borrow_mut`)
the contents of `DATA`.

## `claim*`

If, instead, the sensor tasks had different priorities then the lowest priority task would need to
*claim* (`claim` / `claim_mut`) the resource. `claim*` creates a critical section and grants access
to the contents of a resource for the span of the critical section. To illustrate let's increase the
priority of `accelerometer` to 2; `gyroscope` would then have to access `DATA` like this:

``` rust
fn gyroscope(ctxt: accelerometer::Context) {
    let gyroscope = ctxt.resources.GYROSCOPE;
    let angular_rate = gyroscope.read();

    let t = &mut ctxt.threshold;

    let acceleration = ctxt.resources.DATA.claim_mut(t, |data: &mut Data, _| {
        // start of critical section
        match *data {
            // store data
            Data::Empty => {
                *data = Data::AngularRate(angular_rate);
                None
            },

            // data pair is ready
            Data::Acceleration(acceleration) => {
                *data = Data::Empty;
                Some(acceleration)
            },

            // overwrite old data
            Data::AngularRate(angular_rate) => {
                *data = Data::AngularRate(angular_rate);
                None
            },
        }
        // end of critical section
    });

    if let Some(acceleration) = acceleration {
        ctxt.tasks.process.schedule_now(t, (acceleration, angular_rate)).unwrap();
    }
}
```
