# Software tasks & spawn

The RTIC concept of a software task shares a lot with that of [hardware tasks][hardware_tasks.md]
with the core difference that a software task is not explicitly bound to a specific
interrupt vector, but rather a “dispatcher” interrupt vector running
at the same priority as the software task.

Thus, software tasks are tasks which are not directly assigned to a specific interrupt vector.

The `#[task]` attribute used on a function declare it as a software tasks.
Observe the absence of a `binds = InterruptName` argument to the attribute.
The static method `task_name::spawn()` spawns (starts) a software task and
given that there are no higher priority tasks running the task will start executing directly.

All software tasks at the same priority level shares an interrupt handler acting as a dispatcher.
What differentiates software and hardware tasks are the dispatcher versus bound interrupt vector.

The interrupt vectors used as dispatchers can not be used by hardware tasks.

A list of “free” (not in use by hardware tasks) and usable interrupts allows the framework
to dispatch software tasks.

This list of dispatchers, `dispatchers = [FreeInterrupt1, FreeInterrupt2, ...]` is an
argument to the `#[app]` attribute.

Each interrupt vector acting as dispatcher gets assigned to one priority level meaning that
the list of dispatchers need to cover all priority levels used by software tasks.

Example: The `dispatchers =` argument needs to have at least 3 entries for an application using
three different priorities for software tasks.

The framework will give a compilation error if there are not enough dispatchers provided.

See the following example:

``` rust
{{#include ../../../../examples/spawn.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example spawn
{{#include ../../../../ci/expected/spawn.run}}
```
