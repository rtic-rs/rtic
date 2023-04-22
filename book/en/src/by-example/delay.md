# Tasks with delay

A convenient way to express miniminal timing requirements is by delaying progression. 

This can be achieved by instantiating a monotonic timer (for implementations, see [`rtic-monotonics`]):

[`rtic-monotonics`]: https://github.com/rtic-rs/rtic/tree/master/rtic-monotonics
[`rtic-time`]: https://github.com/rtic-rs/rtic/tree/master/rtic-time

``` rust
...
#[init]
fn init(cx: init::Context) -> (Shared, Local) {
    hprintln!("init");

    let token = rtic_monotonics::create_systick_token!();
    Systick::start(cx.core.SYST, 12_000_000, token);
    ...
```

A *software* task can `await` the delay to expire:

``` rust
#[task]
async fn foo(_cx: foo::Context) {
    ...
    Systick::delay(100.millis()).await;
    ...
}

```

<!-- TODO: move technical explanation to internals -->

Technically, the timer queue is implemented as a list based priority queue, where list-nodes are statically allocated as part of the underlying task `Future`. Thus, the timer queue is infallible at run-time (its size and allocation are determined at compile time).

Similarly the channels implementation, the timer-queue implementation relies on a global *Critical Section* (CS) for race protection. For the examples a CS implementation is provided by adding `--features test-critical-section` to the build options.

<details>
<summary>A complete example</summary>

``` rust
{{#include ../../../../rtic/examples/async-delay.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-delay --features test-critical-section 
```

``` console
{{#include ../../../../rtic/ci/expected/async-delay.run}}
```

</details>

## Timeout

Rust [`Future`]s (underlying Rust `async`/`await`) are composable. This makes it possible to `select` in between `Futures` that have completed.

[`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html

A common use case is transactions with an associated timeout. In the examples shown below, we introduce a fake HAL device that performs some transaction. We have modelled the time it takes based on the input parameter (`n`) as `350ms + n * 100ms`. 

Using the `select_biased` macro from the `futures` crate it may look like this:

``` rust
// Call hal with short relative timeout using `select_biased`
select_biased! {
    v = hal_get(1).fuse() => hprintln!("hal returned {}", v),
    _ = Systick::delay(200.millis()).fuse() =>  hprintln!("timeout", ), // this will finish first
}
```

Assuming the `hal_get` will take 450ms to finish, a short timeout of 200ms will expire before `hal_get` can complete.

Extending the timeout to 1000ms would cause `hal_get` will to complete first.

Using `select_biased` any number of futures can be combined, so its very powerful. However, as the timeout pattern is frequently used, more ergonomic support is baked into RTIC, provided by the [`rtic-monotonics`] and [`rtic-time`] crates. 

Rewriting the second example from above using `timeout_after` gives:

``` rust
// Call hal with long relative timeout using monotonic `timeout_after`
match Systick::timeout_after(1000.millis(), hal_get(1)).await {
    Ok(v) => hprintln!("hal returned {}", v),
    _ => hprintln!("timeout"),
}
```

In cases where you want exact control over time without drift we can use exact points in time using `Instant`, and spans of time using `Duration`. Operations on the `Instant` and `Duration` types come from the [`fugit`] crate.

[fugit]: https://crates.io/crates/fugit

``` rust
// get the current time instance
let mut instant = Systick::now();

// do this 3 times
for n in 0..3 {
    // absolute point in time without drift
    instant += 1000.millis();
    Systick::delay_until(instant).await;

    // absolute point it time for timeout
    let timeout = instant + 500.millis();
    hprintln!("now is {:?}, timeout at {:?}", Systick::now(), timeout);

    match Systick::timeout_at(timeout, hal_get(n)).await {
        Ok(v) => hprintln!("hal returned {} at time {:?}", v, Systick::now()),
        _ => hprintln!("timeout"),
    }
}
```

`let mut instant = Systick::now()` sets the starting time of execution. 

We want to call `hal_get` after 1000ms relative to this starting time. This can be accomplished by using `Systick::delay_until(instant).await`. 

Then, we define a point in time called `timeout`, and call `Systick::timeout_at(timeout, hal_get(n)).await`. 

For the first iteration of the loop, with `n == 0`, the `hal_get` will take 350ms (and finishes before the timeout). 

For the second iteration, with `n == 1`, the `hal_get` will take 450ms (and again succeeds to finish before the timeout).  

For the third iteration, with `n == 2`, `hal_get` will take 550ms to finish, in which case we will run into a timeout.

<details>
<summary>A complete example</summary>

``` rust
{{#include ../../../../rtic/examples/async-timeout.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-timeout --features test-critical-section 
```

``` console
{{#include ../../../../rtic/ci/expected/async-timeout.run}}
```
</details>
