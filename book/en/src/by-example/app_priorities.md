# Task priorities

## Priorities

The `priority` argument declares the static priority of each `task`.

For Cortex-M, tasks can have priorities in the range `0..=(1 << NVIC_PRIO_BITS)` where `NVIC_PRIO_BITS` is a constant defined in the `device` crate.

Omitting the `priority` argument the task priority defaults to `0`. The `idle` task has a non-configurable static priority of `0`, the lowest priority.

> A higher number means a higher priority in RTIC, which is the opposite from what
> Cortex-M does in the NVIC peripheral.
> Explicitly, this means that number `10` has a **higher** priority than number `9`.

The highest static priority task takes precedence when more than one task are ready to execute.

The following scenario demonstrates task prioritization:
Spawning a higher priority task A during execution of a lower priority task B suspends task B. Task A has higher priority thus preempting task B which gets suspended until task A completes execution. Thus, when task A completes task B resumes execution.

```text
Task Priority
  ┌────────────────────────────────────────────────────────┐
  │                                                        │
  │                                                        │
3 │                      Preempts                          │
2 │                    A─────────►                         │
1 │          B─────────► - - - - B────────►                │
0 │Idle┌─────►                   Resumes  ┌──────────►     │
  ├────┴──────────────────────────────────┴────────────────┤
  │                                                        │
  └────────────────────────────────────────────────────────┘Time
```

The following example showcases the priority based scheduling of tasks:

``` rust,noplayground
{{#include ../../../../rtic/examples/preempt.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example preempt
{{#include ../../../../rtic/ci/expected/preempt.run}}
```

Note that the task `bar` does *not* preempt task `baz` because its priority is the *same* as `baz`'s. The higher priority task `bar` runs before `foo` when `baz`returns. When `bar` returns `foo` can resume.

One more note about priorities: choosing a priority higher than what the device supports will result in a compilation error. The error is cryptic due to limitations in the Rust language, if `priority = 9` for task `uart0_interrupt` in `example/common.rs` this looks like:

The error is cryptic due to limitations in the Rust language if `priority = 9` for task `uart0_interrupt` in `example/common.rs` this looks like:

```text
   error[E0080]: evaluation of constant value failed
  --> examples/common.rs:10:1
   |
10 | #[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ attempt to compute `8_usize - 9_usize`, which would overflow
   |
   = note: this error originates in the attribute macro `rtic::app` (in Nightly builds, run with -Z macro-backtrace for more info)

```

The error message incorrectly points to the starting point of the macro, but at least the value subtracted (in this case 9) will suggest which task causes the error.
