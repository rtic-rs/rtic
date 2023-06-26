# RTIC now requires Rust Nightly

The new `async` features require that you use a nightly compiler, and that the feature `type_alias_impl_trait` is enabled for your applications.

To enable this feature, you must add the line `#![feature(type_alias_impl_trait)]` to the root file of your project, on the lines below or above where `#![no_std]` and `#![no_main]` are defined.
