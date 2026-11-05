# ESP32-C6 RTIC template

This crate showcases a simple RTIC application for the ESP32-C6.

## Prerequisites

### Espressif toolchain

This crate uses the most convenient option in ``cargo-espflash`` and ``espflash``

```cargo install cargo-espflash espflash```

## Running the crate

```cargo run --example sw_and_hw (--release)```

should do the trick.

## Expected behavior

The example ``sw_and_hw``

- Prints ``init``,
- Enters a high prio task,
- During the execution of the high prio task, the button should be non-functional,
- Pends a low prio task,
- Exits the high prio task,
- Enters the low prio task,
- During the execution of the low prio task, the button should be functional,
- Exits the low prio task, and
- Prints ``idle``.

The example ``monotonic``

- Prints ``init``,
- Spawns the ``foo``, ``bar``, ``baz`` tasks (because of hardware interrupt latency dispatch, the order here may vary),
- Each task prints ``hello from $TASK`` on entry,
- The tasks wait for 1, 2, 3 seconds respectively, and
- Once the wait period is over, each task exits printing ``bye from $TASK`` (now in the proper order).
