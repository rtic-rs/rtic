# Using indirection for faster message passing

Message passing always involves copying the payload from the sender into a
static variable and then from the static variable into the receiver. Thus
sending a large buffer, like a `[u8; 128]`, as a message involves two expensive
`memcpy`s.

Indirection can minimize message passing overhead:
instead of sending the buffer by value, one can send an owning pointer into the
buffer.

One can use a global allocator to achieve indirection (`alloc::Box`,
`alloc::Rc`, etc.), which requires using the nightly channel as of Rust v1.37.0,
or one can use a statically allocated memory pool like [`heapless::Pool`].

[`heapless::Pool`]: https://docs.rs/heapless/0.5.0/heapless/pool/index.html

Here's an example where `heapless::Pool` is used to "box" buffers of 128 bytes.

``` rust
{{#include ../../../../examples/pool.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example pool
{{#include ../../../../ci/expected/pool.run}}
```
