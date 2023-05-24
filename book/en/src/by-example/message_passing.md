# Message passing & capacity

Software tasks support message passing, this means that software tasks can be spawned
with an argument: `foo::spawn(1)` which will run the task `foo` with the argument `1`.

Capacity sets the size of the spawn queue for the task. If it is not specified, the capacity defaults to 1.

In the example below, the capacity of task `foo` is `3`, allowing three simultaneous
pending spawns of `foo`. Exceeding this capacity is an `Error`.

The number of arguments to a task is not limited:

``` rust,noplayground
{{#include ../../../../examples/message_passing.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example message_passing
{{#include ../../../../ci/expected/message_passing.run}}
```
