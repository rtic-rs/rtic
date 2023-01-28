# Tasks with delay

A convenient way to express *miniminal* timing requirements is by means of delaying progression. 

This can be achieved by instantiating a monotonic timer:

```rust
...
rtic_monotonics::make_systick_timer_queue!(TIMER);

#[init]
fn init(cx: init::Context) -> (Shared, Local) {        
    let systick = Systick::start(cx.core.SYST, 12_000_000);
    TIMER.initialize(systick);
    ... 
```

A *software* task can `await` the delay to expire:

```rust
#[task]
async fn foo(_cx: foo::Context) {
    ...
    TIMER.delay(100.millis()).await;
    ...
```

Technically, the timer queue is implemented as a list based priority queue, where list-nodes are statically allocated as part of the underlying task `Future`. Thus, the timer queue is infallible at run-time (its size and allocation is determined at compile time).

For a complete example:

``` rust
{{#include ../../../../rtic/examples/async-delay.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-delay --features test-critical-section 
{{#include ../../../../rtic/ci/expected/async-delay.run}}
```

## Timeout

Rust `Futures` (underlying Rust `async`/`await`) are composable. This makes it possible to `select` in between `Futures` that have completed.

A common use case is transactions with associated timeout. In the examples shown below, we introduce a fake HAL device which performs some transaction. We have modelled the time it takes based on the input parameter (`n`) as `350ms + n * 100ms)`. 

Using the `select_biased` macro from the `futures` crate it may look like this:

```rust
// Call hal with short relative timeout using `select_biased`
select_biased! {
    v = hal_get(&TIMER, 1).fuse() => hprintln!("hal returned {}", v),
    _ = TIMER.delay(200.millis()).fuse() =>  hprintln!("timeout", ), // this will finish first
}
```

Assuming the `hal_get` will take 450ms to finish, a short timeout of 200ms will expire.

```rust
// Call hal with long relative timeout using `select_biased`
select_biased! {
    v = hal_get(&TIMER, 1).fuse() => hprintln!("hal returned {}", v), // hal finish first
    _ = TIMER.delay(1000.millis()).fuse() =>  hprintln!("timeout", ),
}
```

By extending the timeout to 1000ms, the `hal_get` will finish first.

Using `select_biased` any number of futures can be combined, so its very powerful. However, as the timeout pattern is frequently used, it is directly supported by the RTIC [rtc-monotonics] and [rtic-time] crates. The second example from above using `timeout_after`:

```rust
// Call hal with long relative timeout using monotonic `timeout_after`
match TIMER.timeout_after(1000.millis(), hal_get(&TIMER, 1)).await {
    Ok(v) => hprintln!("hal returned {}", v),
    _ => hprintln!("timeout"),
}
```

In cases you want exact control over time without drift. For this purpose we can use exact points in time using `Instance`, and spans of time using `Duration`. Operations on the `Instance` and `Duration` types are given by the [fugit] crate.

[fugit]: https://crates.io/crates/fugit

```rust
// get the current time instance
let mut instant = TIMER.now();

// do this 3 times
for n in 0..3 {
    // exact point in time without drift
    instant += 1000.millis();
    TIMER.delay_until(instant).await;

    // exact point it time for timeout
    let timeout = instant + 500.millis();
    hprintln!("now is {:?}, timeout at {:?}", TIMER.now(), timeout);

    match TIMER.timeout_at(timeout, hal_get(&TIMER, n)).await {
        Ok(v) => hprintln!("hal returned {} at time {:?}", v, TIMER.now()),
        _ => hprintln!("timeout"),
    }
}
```

`instant = TIMER.now()` gives the baseline (i.e., the exact current point in time). We want to call `hal_get` after 1000ms relative to this exact point in time. This can be accomplished by `TIMER.delay_until(instant).await;`. We define the absolute point in time for the `timeout`, and call `TIMER.timeout_at(timeout, hal_get(&TIMER, n)).await`. For the first loop iteration `n == 0`, and the `hal_get` will take 350ms (and finishes before the timeout). For the second iteration `n == 1`, and `hal_get` will take 450ms (and again succeeds to finish before the timeout).  For the third iteration `n == 2` (`hal_get` will take 5500ms to finish). In this case we will run into a timeout.


The complete example:

``` rust
{{#include ../../../../rtic/examples/async-timeout.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example async-timeout --features test-critical-section 
{{#include ../../../../rtic/ci/expected/async-timeout.run}}
```
