# Summary

[Preface](./preface.md)

- [Starting a new project](./starting_a_project.md)
- [RTIC by example](./by-example.md)
  - [The `app`](./by-example/app.md)
  - [Hardware tasks & `pend`](./by-example/hardware_tasks.md)
  - [Software tasks & `spawn`](./by-example/software_tasks.md)
  - [Resources](./by-example/resources.md)
  - [The init task](./by-example/app_init.md)
  - [The idle task](./by-example/app_idle.md)
  - [Channel based communication](./by-example/channel.md)
  - [Delay and Timeout using Monotonics](./by-example/delay.md)
  - [The minimal app](./by-example/app_minimal.md)
  - [Tips & Tricks](./by-example/tips.md)
    - [Implementing Monotonic](./by-example/tips_monotonic_impl.md)
    - [Resource de-structure-ing](./by-example/tips_destructureing.md)
    - [Avoid copies when message passing](./by-example/tips_indirection.md)
    - [`'static` super-powers](./by-example/tips_static_lifetimes.md)
    - [Inspecting generated code](./by-example/tips_view_code.md)
    <!-- - [Running tasks from RAM](./by-example/tips_from_ram.md) -->
    <!-- - [`#[cfg(..)]` support](./by-example/tips.md) -->
- [RTIC vs. the world](./rtic_vs.md)
- [Awesome RTIC examples](./awesome_rtic.md)
- [Migrating from v1.0.x to v2.0.0](./migration_v1_v2.md)
  - [Rust Nightly & features](./migration_v1_v2/nightly.md)
  - [Migrating to `rtic-monotonics`](./migration_v1_v2/monotonics.md)
  - [Software tasks must now be `async`](./migration_v1_v2/async_tasks.md)
  - [Using and understanding `rtic-sync`](./migration_v1_v2/rtic-sync.md)
  - [A code example on migration](./migration_v1_v2/complete_example.md)
- [Under the hood](./internals.md)
  - [Cortex-M architectures](./internals/targets.md)
  <!--- [Interrupt configuration](./internals/interrupt-configuration.md)-->
  <!--- [Non-reentrancy](./internals/non-reentrancy.md)-->
  <!--- [Access control](./internals/access.md)-->
  <!--- [Late resources](./internals/late-resources.md)-->
  <!--- [Critical sections](./internals/critical-sections.md)-->
  <!--- [Ceiling analysis](./internals/ceilings.md)-->
  <!--- [Software tasks](./internals/tasks.md)-->
  <!--- [Timer queue](./internals/timer-queue.md)-->

  <!-- - [Defining tasks](./by-example/app_task.md) -->
  <!-- - [Software tasks & `spawn`](./by-example/software_tasks.md)
    - [Message passing & `capacity`](./by-example/message_passing.md)
    - [Task priorities](./by-example/app_priorities.md)
    - [Monotonic & `spawn_{at/after}`](./by-example/monotonic.md) 
  -->