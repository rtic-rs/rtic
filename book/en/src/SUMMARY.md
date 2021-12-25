# Summary

[Preface](./preface.md)

- [RTIC by example](./by-example.md)
  - [The `app`](./by-example/app.md)
  - [Resources](./by-example/resources.md)
  - [The init task](./by-example/app_init.md)
  - [The idle task](./by-example/app_idle.md)
  - [Defining tasks](./by-example/app_task.md)
    - [Hardware tasks](./by-example/hardware_tasks.md)
    - [Software tasks & `spawn`](./by-example/software_tasks.md)
    - [Message passing & `capacity`](./by-example/message_passing.md)
    - [Task priorities](./by-example/app_priorities.md)
    - [Monotonic & `spawn_{at/after}`](./by-example/monotonic.md)
  - [Starting a new project](./by-example/starting_a_project.md)
  - [The minimal app](./by-example/app_minimal.md)
  - [Tips & Tricks](./by-example/tips.md)
    - [Implementing Monotonic](./by-example/tips_monotonic_impl.md)
    - [Resource de-structure-ing](./by-example/tips_destructureing.md)
    - [Avoid copies when message passing](./by-example/tips_indirection.md)
    - [`'static` super-powers](./by-example/tips_static_lifetimes.md)
    - [Inspecting generated code](./by-example/tips_view_code.md)
    - [Running tasks from RAM](./by-example/tips_from_ram.md)
    <!-- - [`#[cfg(..)]` support](./by-example/tips.md) -->
- [Awesome RTIC examples](./awesome_rtic.md)
- [Migration Guides](./migration.md)
  - [v0.5.x to v1.0.x](./migration/migration_v5.md)
  - [v0.4.x to v0.5.x](./migration/migration_v4.md)
  - [RTFM to RTIC](./migration/migration_rtic.md)
- [Under the hood](./internals.md)
  <!--- [Interrupt configuration](./internals/interrupt-configuration.md)-->
  <!--- [Non-reentrancy](./internals/non-reentrancy.md)-->
  <!--- [Access control](./internals/access.md)-->
  <!--- [Late resources](./internals/late-resources.md)-->
  <!--- [Critical sections](./internals/critical-sections.md)-->
  <!--- [Ceiling analysis](./internals/ceilings.md)-->
  <!--- [Software tasks](./internals/tasks.md)-->
  <!--- [Timer queue](./internals/timer-queue.md)-->
