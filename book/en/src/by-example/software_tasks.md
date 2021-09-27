# Software tasks & spawn

Software tasks, as hardware tasks, are run as interrupt handlers where all software tasks at the
same priority shares a "free" interrupt handler to run from, called a dispatcher. These free
interrupts are interrupt vectors not used by hardware tasks.

To declare tasks in the framework the `#[task]` attribute is used on a function.
By default these tasks are referred to as software tasks as they do not have a direct coupling to
an interrupt handler. Software tasks can be spawned (started) using the `task_name::spawn()` static
method which will directly run the task given that there are no higher priority tasks running.

To indicate to the framework which interrupts are free for use to dispatch software tasks with the
`#[app]` attribute has a `dispatchers = [FreeInterrupt1, FreeInterrupt2, ...]` argument. You need
to provide as many dispatchers as there are priority levels used by software tasks, as an
dispatcher is assigned per interrupt level. The framework will also give a compile error if there
are not enough dispatchers provided.

This is exemplified in the following:

``` rust
{{#include ../../../../examples/spawn.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example spawn
{{#include ../../../../ci/expected/spawn.run}}
```
