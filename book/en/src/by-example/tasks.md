# Software tasks

In addition to hardware tasks, which are invoked by the hardware in response to
hardware events, RTFM also supports *software* tasks which can be spawned by the
application from any execution context.

Software tasks can also be assigned priorities and, under the hood, are
dispatched from interrupt handlers. RTFM requires that free interrupts are
declared in an `extern` block when using software tasks; some of these free
interrupts will be used to dispatch the software tasks. An advantage of software
tasks over hardware tasks is that many tasks can be mapped to a single interrupt
handler.

Software tasks are also declared using the `task` attribute but the `binds`
argument must be omitted. To be able to spawn a software task from a context
the name of the task must appear in the `spawn` argument of the context
attribute (`init`, `idle`, `task`, etc.).

The example below showcases three software tasks that run at 2 different
priorities. The three software tasks are mapped to 2 interrupts handlers.

``` rust
{{#include ../../../../examples/task.rs}}
```

``` console
$ cargo run --example task
{{#include ../../../../ci/expected/task.run}}```

## Message passing

The other advantage of software tasks is that messages can be passed to these
tasks when spawning them. The type of the message payload must be specified in
the signature of the task handler.

The example below showcases three tasks, two of them expect a message.

``` rust
{{#include ../../../../examples/message.rs}}
```

``` console
$ cargo run --example message
{{#include ../../../../ci/expected/message.run}}```

## Capacity

RTFM does *not* perform any form of heap-based memory allocation. The memory
required to store messages is statically reserved. By default the framework
minimizes the memory footprint of the application so each task has a message
"capacity" of 1: meaning that at most one message can be posted to the task
before it gets a chance to run. This default can be overridden for each task
using the `capacity` argument. This argument takes a positive integer that
indicates how many messages the task message buffer can hold.

The example below sets the capacity of the software task `foo` to 4. If the
capacity is not specified then the second `spawn.foo` call in `UART0` would
fail (panic).

``` rust
{{#include ../../../../examples/capacity.rs}}
```

``` console
$ cargo run --example capacity
{{#include ../../../../ci/expected/capacity.run}}```

## Error handling

The `spawn` API returns the `Err` variant when there's no space to send the
message. In most scenarios spawning errors are handled in one of two ways:

- Panicking, using `unwrap`, `expect`, etc. This approach is used to catch the
  programmer   error (i.e. bug) of selecting a capacity that was too small. When
  this panic is encountered during testing choosing a bigger capacity and
  recompiling the program may fix the issue but sometimes it's necessary to dig
  deeper and perform a timing analysis of the application to check if the
  platform can deal with peak payload or if the processor needs to be replaced
  with a faster one.

- Ignoring the result. In soft real time and non real time applications it may
  be OK to occasionally lose data or fail to respond to some events during event
  bursts. In those scenarios silently letting a `spawn` call fail may be
  acceptable.

It should be noted that retrying a `spawn` call is usually the wrong approach as
this operation will likely never succeed in practice. Because there are only
context switches towards *higher* priority tasks retrying the `spawn` call of a
lower priority task will never let the scheduler dispatch said task meaning that
its message buffer will never be emptied. This situation is depicted in the
following snippet:

``` rust
#[rtfm::app(..)]
const APP: () = {
    #[init(spawn = [foo, bar])]
    fn init(cx: init::Context) {
        cx.spawn.foo().unwrap();
        cx.spawn.bar().unwrap();
    }

    #[task(priority = 2, spawn = [bar])]
    fn foo(cx: foo::Context) {
        // ..

        // the program will get stuck here
        while cx.spawn.bar(payload).is_err() {
            // retry the spawn call if it failed
        }
    }

    #[task(priority = 1)]
    fn bar(cx: bar::Context, payload: i32) {
        // ..
    }
};
```
