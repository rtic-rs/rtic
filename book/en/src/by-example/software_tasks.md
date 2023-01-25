# Software tasks & spawn

The RTIC concept of a software task shares a lot with that of [hardware tasks](./hardware_tasks.md)
with the core difference that a software task is not explicitly bound to a specific
interrupt vector, but rather bound to a “dispatcher” interrupt vector running
at the intended priority of the software task (see below).

Thus, software tasks are tasks which are not *directly* bound to an interrupt vector.

The `#[task]` attributes used on a function determine if it is
software tasks, specifically the absence of a `binds = InterruptName`
argument to the attribute definition.

The static method `task_name::spawn()` spawns (schedules) a software
task by registering it with a specific dispatcher.  If there are no
higher priority tasks available to the scheduler (which serves a set
of dispatchers), the task will start executing directly.

All software tasks at the same priority level share an interrupt handler bound to their dispatcher.
What differentiates software and hardware tasks is the usage of either a dispatcher or a bound interrupt vector.

The interrupt vectors used as dispatchers cannot be used by hardware tasks.

Availability of a set of “free” (not in use by hardware tasks) and usable interrupt vectors allows the framework
to dispatch software tasks via dedicated interrupt handlers.

This set of dispatchers, `dispatchers = [FreeInterrupt1, FreeInterrupt2, ...]` is an
argument to the `#[app]` attribute.

Each interrupt vector acting as dispatcher gets assigned to a unique priority level meaning that
the list of dispatchers needs to cover all priority levels used by software tasks.

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
