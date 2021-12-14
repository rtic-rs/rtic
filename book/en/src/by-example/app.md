# The `#[app]` attribute and an RTIC application

## Requirements on the `app` attribute

All RTIC applications use the [`app`] attribute (`#[app(..)]`). This attribute
only applies to a `mod`-item containing the RTIC application. The `app`
attribute has a mandatory `device` argument that takes a *path* as a value.
This must be a full path pointing to a
*peripheral access crate* (PAC) generated using [`svd2rust`] **v0.14.x** or
newer.

The `app` attribute will expand into a suitable entry point and thus replaces
the use of the [`cortex_m_rt::entry`] attribute.

[`app`]: ../../../api/cortex_m_rtic_macros/attr.app.html
[`svd2rust`]: https://crates.io/crates/svd2rust
[`cortex_m_rt::entry`]: ../../../api/cortex_m_rt_macros/attr.entry.html

## An RTIC application example

To give a flavour of RTIC, the following example contains commonly used features.
In the following sections we will go through each feature in detail.

``` rust
{{#include ../../../../examples/common.rs}}
```
