# Software tasks & spawn

To declare tasks in the framework the `#[task]` attribute is used on a function.
By default these tasks are referred to as software tasks as they do not have a direct coupling to
an interrupt handler. Software tasks can be spawned (started) using the `task_name::spawn()` static
method which will directly run the task given that there are no higher priority tasks running.
This is exemplified in the following:

``` rust
{{#include ../../../../examples/spawn.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example spawn
{{#include ../../../../ci/expected/spawn.run}}
```
