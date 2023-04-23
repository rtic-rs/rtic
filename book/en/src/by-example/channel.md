# Communication over channels.

Channels can be used to communicate data between running tasks. The channel is essentially a wait queue, allowing tasks with multiple producers and a single receiver. A channel is constructed in the `init` task and backed by statically allocated memory. Send and receive endpoints are distributed to *software* tasks:

``` rust,noplayground
...
const CAPACITY: usize = 5;
#[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let (s, r) = make_channel!(u32, CAPACITY);
        receiver::spawn(r).unwrap();
        sender1::spawn(s.clone()).unwrap();
        sender2::spawn(s.clone()).unwrap();
        ...
```

In this case the channel holds data of `u32` type with a capacity of 5  elements. 

Channels can also be used from *hardware* tasks, but only in a non-`async` manner using the [Try API](#try-api).

## Sending data

The `send` method post a message on the channel as shown below:

``` rust,noplayground
#[task]
async fn sender1(_c: sender1::Context, mut sender: Sender<'static, u32, CAPACITY>) {
    hprintln!("Sender 1 sending: 1");
    sender.send(1).await.unwrap();
}
```

## Receiving data

The receiver can `await` incoming messages:

``` rust,noplayground
#[task]
async fn receiver(_c: receiver::Context, mut receiver: Receiver<'static, u32, CAPACITY>) {
    while let Ok(val) = receiver.recv().await {
        hprintln!("Receiver got: {}", val);
        ...
    }
}
```

Channels are implemented using a small (global) *Critical Section* (CS) for protection against race-conditions. The user must provide an CS implementation. Compiling the examples given the `--features test-critical-section` gives one possible implementation. 

For a complete example:

``` rust,noplayground
{{#include ../../../../rtic/examples/async-channel.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-channel --features test-critical-section 
```

``` console
{{#include ../../../../rtic/ci/expected/async-channel.run}}
```

Also sender endpoint can be awaited. In case the channel capacity has not yet been reached, `await`-ing the sender can progress immediately, while in the case the capacity is reached, the sender is blocked until there is free space in the queue. In this way data is never lost.

In the following example the `CAPACITY` has been reduced to 1, forcing sender tasks to wait until the data in the channel has been received.

``` rust,noplayground
{{#include ../../../../rtic/examples/async-channel-done.rs}}
```

Looking at the output, we find that `Sender 2` will wait until the data sent by `Sender 1` as been received. 

> **NOTICE** *Software* tasks at the same priority are executed asynchronously to each other, thus **NO** strict order can be assumed. (The presented order here applies only to the current implementation, and may change between RTIC framework releases.)

``` console
$ cargo run --target thumbv7m-none-eabi --example async-channel-done --features test-critical-section 
{{#include ../../../../rtic/ci/expected/async-channel-done.run}}
```

## Error handling

In case all senders have been dropped `await`-ing on an empty receiver channel results in an error. This allows to gracefully implement different types of shutdown operations.

``` rust,noplayground
{{#include ../../../../rtic/examples/async-channel-no-sender.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-channel-no-sender --features test-critical-section 
```

``` console 
{{#include ../../../../rtic/ci/expected/async-channel-no-sender.run}}
```

Similarly, `await`-ing on a send channel results in an error in case the receiver has been dropped. This allows to gracefully implement application level error handling.

The resulting error returns the data back to the sender, allowing the sender to take appropriate action (e.g., storing the data to later retry sending it).

``` rust,noplayground
{{#include ../../../../rtic/examples/async-channel-no-receiver.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-channel-no-receiver --features test-critical-section 
```

``` console
{{#include ../../../../rtic/ci/expected/async-channel-no-receiver.run}}
```

## Try API

Using the Try API, you can send or receive data from or to a channel without requiring that the operation succeeds, and in non-`async` contexts.

This API is exposed through `Receiver::try_recv` and `Sender::try_send`.

``` rust,noplayground
{{#include ../../../../rtic/examples/async-channel-try.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-channel-try --features test-critical-section
```

``` console 
{{#include ../../../../rtic/ci/expected/async-channel-try.run}}
```