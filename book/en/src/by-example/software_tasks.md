# Software tasks & spawn

Software tasks are tasks which are not directly assigned to a specific interrupt vector.

They run as interrupt handlers where all software tasks at the
same priority level shares a "free" interrupt handler acting as a dispatcher.
Thus, what differentiates software and hardware tasks are the dispatcher versus
bound interrupt vector.

These free interrupts used as dispatchers are interrupt vectors not used by hardware tasks.

The `#[task]` attribute used on a function declare it as a software tasks.
The static method `task_name::spawn()` spawn (start) a software task and
given that there are no higher priority tasks running the task will start executing directly.

A list of “free” and usable interrupts allows the framework to dispatch software tasks.
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
