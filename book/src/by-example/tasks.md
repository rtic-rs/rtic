# Software tasks

RTFM treats interrupt and exception handlers as *hardware* tasks. Hardware tasks
are invoked by the hardware in response to events, like pressing a button. RTFM
also supports *software* tasks which can be spawned by the software from any
execution context.

Software tasks can also be assigned priorities and are dispatched from interrupt
handlers. RTFM requires that free interrupts are declared in an `extern` block
when using software tasks; these free interrupts will be used to dispatch the
software tasks. An advantage of software tasks over hardware tasks is that many
tasks can be mapped to a single interrupt handler.

Software tasks are declared by applying the `task` attribute to functions. To be
able to spawn a software task the name of the task must appear in the `spawn`
argument of the context attribute (`init`, `idle`, `interrupt`, etc.).

The example below showcases three software tasks that run at 2 different
priorities. The three tasks map to 2 interrupts handlers.

``` rust
{{#include ../../../examples/task.rs}}
```

``` console
$ cargo run --example task
{{#include ../../../ci/expected/task.run}}```

## Message passing

The other advantage of software tasks is that messages can be passed to these
tasks when spawning them. The type of the message payload must be specified in
the signature of the task handler.

The example below showcases three tasks, two of them expect a message.

``` rust
{{#include ../../../examples/message.rs}}
```

``` console
$ cargo run --example message
{{#include ../../../ci/expected/message.run}}```

## Capacity

Task dispatchers do *not* use any dynamic memory allocation. The memory required
to store messages is statically reserved. The framework will reserve enough
space for every context to be able to spawn each task at most once. This is a
sensible default but the "inbox" capacity of each task can be controlled using
the `capacity` argument of the `task` attribute.

The example below sets the capacity of the software task `foo` to 4. If the
capacity is not specified then the second `spawn.foo` call in `UART0` would
fail.

``` rust
{{#include ../../../examples/capacity.rs}}
```

``` console
$ cargo run --example capacity
{{#include ../../../ci/expected/capacity.run}}```
