# 'static super-powers

As discussed earlier `local` resources are given `'static` lifetime in `#[init]` and `#[idle]`,
this can be used to allocate an object and then split it up or give the pre-allocated object to a
task, driver or some other object.
This is very useful when needing to allocate memory for drivers, such as USB drivers, and using
data structures that can be split such as [`heapless::spsc::Queue`].

In the following example an [`heapless::spsc::Queue`] is given to two different tasks for lock-free access
to the shared queue.

[`heapless::spsc::Queue`]: https://docs.rs/heapless/0.7.5/heapless/spsc/struct.Queue.html


``` rust
{{#include ../../../../examples/static.rs}}
```

Running this program produces the expected output.

``` console
$ cargo run --target thumbv7m-none-eabi --example static
{{#include ../../../../ci/expected/static.run}}
```
