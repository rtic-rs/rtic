# The `#[app]` attribute and an RTIC application

## Requirements on the `app` attribute

All RTIC applications use the [`app`] attribute (`#[app(..)]`). This attribute only applies to a `mod`-item containing the RTIC application.

The `app` attribute has a mandatory `device` argument that takes a *path* as a value. This must be a full path pointing to a *peripheral access crate* (PAC) generated using [`svd2rust`] **v0.14.x** or newer.

The `app` attribute will expand into a suitable entry point and thus replaces the use of the [`cortex_m_rt::entry`] attribute.

[`app`]: ../../../api/rtic_macros/attr.app.html
[`svd2rust`]: https://crates.io/crates/svd2rust
[`cortex_m_rt::entry`]: https://docs.rs/cortex-m-rt-macros/latest/cortex_m_rt_macros/attr.entry.html

## Structure and zero-cost concurrency

An RTIC `app` is an executable system model for single-core applications, declaring a set of `local` and `shared` resources operated on by a set of `init`, `idle`, *hardware* and *software* tasks. 

* `init`  runs before any other task, and returns the `local` and `shared` resources.
* Tasks (both hardware and software) run preemptively based on their associated static priority.
* Hardware tasks are bound to underlying hardware interrupts.
* Software tasks are schedulied by an set of asynchronous executors, one for each software task priority.
* `idle` has the lowest priority, and can be used for background work, and/or to put the system to sleep until it is woken by some event.

At compile time the task/resource model is analyzed under the Stack Resource Policy (SRP) and executable code generated with the following outstanding properties:

- Guaranteed race-free resource access and deadlock-free execution on a single-shared stack.
- Hardware task scheduling is performed directly by the hardware.
- Software task scheduling is performed by auto generated async executors tailored to the application.

Overall, the generated code infers no additional overhead in comparison to a hand-written implementation, thus in Rust terms RTIC offers a zero-cost abstraction to concurrency.

## Priority

Priorities in RTIC are specified using the `priority = N` (where N is a positive number) argument passed to the `#[task]` attribute. All `#[task]`s can have a priority. If the priority of a task is not specified, it is set to the default value of 0.

Priorities in RTIC follow a higher value = more important scheme. For examples, a task with priority 2 will preempt a task with priority 1.

## An RTIC application example

To give a taste of RTIC, the following example contains commonly used features.
In the following sections we will go through each feature in detail.

``` rust,noplayground
{{#include ../../../../rtic/examples/common.rs}}
```
