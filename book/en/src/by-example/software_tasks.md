# Software tasks & spawn

The RTIC concept of a software task shares a lot with that of [hardware tasks](./hardware_tasks.md). The core difference is that a software task is not explicitly bound to a specific interrupt vector, but rather bound to a “dispatcher” interrupt vector running at the intended priority of the software task (see below).

Similarly to _hardware_ tasks, the `#[task]` attribute used on a function declare it as a task. The absence of a `binds = InterruptName` argument to the attribute declares the function as a _software task_.

The static method `task_name::spawn()` spawns (starts) a software task and given that there are no higher priority tasks running the task will start executing directly.

The _software_ task itself is given as an `async` Rust function, which allows the user to optionally `await` future events. This allows to blend reactive programming (by means of _hardware_ tasks) with sequential programming (by means of _software_ tasks).

While _hardware_ tasks are assumed to run-to-completion (and return), _software_ tasks may be started (`spawned`) once and run forever, on the condition that any loop (execution path) is broken by at least one `await` (yielding operation).

## Dispatchers

All _software_ tasks at the same priority level share an interrupt handler acting as an async executor dispatching the software tasks. This list of dispatchers, `dispatchers = [FreeInterrupt1, FreeInterrupt2, ...]` is an argument to the `#[app]` attribute, where you define the set of free and usable interrupts.

Each interrupt vector acting as dispatcher gets assigned to one priority level meaning that the list of dispatchers need to cover all priority levels used by software tasks.

Example: The `dispatchers =` argument needs to have at least 3 entries for an application using three different priorities for software tasks.

The framework will give a compilation error if there are not enough dispatchers provided, or if a clash occurs between the list of dispatchers and interrupts bound to _hardware_ tasks.

See the following example:

```rust,noplayground
{{#include ../../../../examples/lm3s6965/examples/spawn.rs}}
```

```console
$ cargo xtask qemu --verbose --example spawn
```

```console
{{#include ../../../../ci/expected/lm3s6965/spawn.run}}
```

You may `spawn` a _software_ task again, given that it has run-to-completion (returned).

In the below example, we `spawn` the _software_ task `foo` from the `idle` task. Since the priority of the _software_ task is 1 (higher than `idle`), the dispatcher will execute `foo` (preempting `idle`). Since `foo` runs-to-completion. It is ok to `spawn` the `foo` task again.

Technically the async executor will `poll` the `foo` _future_ which in this case leaves the _future_ in a _completed_ state.

```rust,noplayground
{{#include ../../../../examples/lm3s6965/examples/spawn_loop.rs}}
```

```console
$ cargo xtask qemu --verbose --example spawn_loop
```

```console
{{#include ../../../../ci/expected/lm3s6965/spawn_loop.run}}
```

An attempt to `spawn` an already spawned task (running) task will result in an error. Notice, the that the error is reported before the `foo` task is actually run. This is since, the actual execution of the _software_ task is handled by the dispatcher interrupt (`SSIO`), which is not enabled until we exit the `init` task. (Remember, `init` runs in a critical section, i.e. all interrupts being disabled.)

Technically, a `spawn` to a _future_ that is not in _completed_ state is considered an error.

```rust,noplayground
{{#include ../../../../examples/lm3s6965/examples/spawn_err.rs}}
```

```console
$ cargo xtask qemu --verbose --example spawn_err
```

```console
{{#include ../../../../ci/expected/lm3s6965/spawn_err.run}}
```

## Passing arguments

You can also pass arguments at spawn as follows.

```rust,noplayground
{{#include ../../../../examples/lm3s6965/examples/spawn_arguments.rs}}
```

```console
$ cargo xtask qemu --verbose --example spawn_arguments
```

```console
{{#include ../../../../ci/expected/lm3s6965/spawn_arguments.run}}
```

## Divergent tasks

A task can have one of two signatures: `async fn({name}::Context, ..)` or `async fn({name}::Context, ..) -> !`. The latter defines a *divergent* task — one that never returns. The key advantage of divergent tasks is that they receive a `'static` context, and `local` resources have `'static` lifetime. Additionally, using this signature makes the task’s intent explicit, clearly distinguishing between short-lived tasks and those that run indefinitely.

## Priority zero tasks

In RTIC tasks run preemptively to each other, with priority zero (0) the lowest priority. You can use priority zero tasks for background work, without any strict real-time requirements.

Conceptually, one can see such tasks as running in the `main` thread of the application, thus the resources associated are not required the [Send] bound.

[Send]: https://doc.rust-lang.org/nomicon/send-and-sync.html

```rust,noplayground
{{#include ../../../../examples/lm3s6965/examples/zero-prio-task.rs}}
```

```console
$ cargo xtask qemu --verbose --example zero-prio-task
```

```console
{{#include ../../../../ci/expected/lm3s6965/zero-prio-task.run}}
```

> **Notice**: _software_ task at zero priority cannot co-exist with the [idle] task. The reason is that `idle` is running as a non-returning Rust function at priority zero. Thus there would be no way for an executor at priority zero to give control to _software_ tasks at the same priority.

---

Application side safety: Technically, the RTIC framework ensures that `poll` is never executed on any _software_ task with _completed_ future, thus adhering to the soundness rules of async Rust.
