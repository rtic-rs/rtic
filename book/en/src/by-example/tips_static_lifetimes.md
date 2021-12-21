# 'static super-powers

In `#[init]` and `#[idle]` `local` resources has `'static` lifetime.

Useful when pre-allocating and/or splitting resources between tasks, drivers
or some other object.
This comes in handy when drivers, such as USB drivers, need to allocate memory and
when using splittable data structures such as [`heapless::spsc::Queue`].

In the following example two different tasks share a [`heapless::spsc::Queue`]
for lock-free access to the shared queue.

[`heapless::spsc::Queue`]: https://docs.rs/heapless/0.7.5/heapless/spsc/struct.Queue.html

``` rust
{{#include ../../../../examples/static.rs}}
```

Running this program produces the expected output.

``` console
$ cargo run --target thumbv7m-none-eabi --example static
{{#include ../../../../ci/expected/static.run}}
```
