# Using `async` softare tasks.

There have been a few changes to software tasks. They are outlined below.

### Software tasks must now be `async`.

All software tasks are now required to be `async`.

#### Required changes.

All of the tasks in your project that do not bind to an interrupt must now be an `async fn`. For example:

``` rust,noplayground
#[task(
    local = [ some_resource ],
    shared = [ my_shared_resource ],
    priority = 2
)]
fn my_task(cx: my_task::Context) {
    cx.local.some_resource.do_trick();
    cx.shared.my_shared_resource.lock(|s| s.do_shared_thing());
}
```

becomes

``` rust,noplayground
#[task(
    local = [ some_resource ],
    shared = [ my_shared_resource ],
    priority = 2
)]
async fn my_task(cx: my_task::Context) {
    cx.local.some_resource.do_trick();
    cx.shared.my_shared_resource.lock(|s| s.do_shared_thing());
}
```

## Software tasks may now run forever

The new `async` software tasks are allowed to run forever, on one precondition: **there must be an `await` within the infinite loop of the task**. An example of such a task:

``` rust,noplayground
#[task(local = [ my_channel ] )]
async fn my_task_that_runs_forever(cx: my_task_that_runs_forever::Context) {
    loop {
        let value = cx.local.my_channel.recv().await;
        do_something_with_value(value);
    }
}
```

## `spawn_after` and `spawn_at` have been removed.

As discussed in the [Migrating to `rtic-monotonics`](./monotonics.md) chapter, `spawn_after` and `spawn_at` are no longer available.