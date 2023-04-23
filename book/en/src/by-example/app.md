# The `#[app]` attribute and an RTIC application

## Requirements on the `app` attribute

All RTIC applications use the [`app`] attribute (`#[app(..)]`). This attribute only applies to a `mod`-item containing the RTIC application. The `app` attribute has a mandatory `device` argument that takes a *path* as a value. This must be a full path pointing to a *peripheral access crate* (PAC) generated using [`svd2rust`] **v0.14.x** or newer.

The `app` attribute will expand into a suitable entry point and thus replaces the use of the [`cortex_m_rt::entry`] attribute.

[`app`]: ../../../api/rtic_macros/attr.app.html
[`svd2rust`]: https://crates.io/crates/svd2rust
[`cortex_m_rt::entry`]: https://docs.rs/cortex-m-rt-macros/latest/cortex_m_rt_macros/attr.entry.html

## Structure and zero-cost concurrency

An RTIC `app` is an executable system model for single-core applications, declaring a set of `local` and `shared` resources operated on by a set of `init`, `idle`, *hardware* and *software* tasks. In short the `init` task runs before any other task returning the set of `local` and `shared` resources. Tasks run preemptively based on their associated static priority, `idle` has the lowest priority (and can be used for background work, and/or to put the system to sleep until woken by some event). Hardware tasks are bound to underlying hardware interrupts, while software tasks are scheduled by asynchronous executors (one for each software task priority). 

At compile time the task/resource model is analyzed under the Stack Resource Policy (SRP) and executable code generated with the following outstanding properties:

- guaranteed race-free resource access and deadlock-free execution on a single-shared stack
  - hardware task scheduling is performed directly by the hardware, and
  - software task scheduling is performed by auto generated async executors tailored to the application.

Overall, the generated code infers no additional overhead in comparison to a hand-written implementation, thus in Rust terms RTIC offers a zero-cost abstraction to concurrency.

## An RTIC application example

To give a flavour of RTIC, the following example contains commonly used features.
In the following sections we will go through each feature in detail.

``` rust,noplayground
{{#include ../../../../rtic/examples/common.rs}}
```
