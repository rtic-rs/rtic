//! Minimal example with zero tasks
//!
//! ```
//! 
//! #![deny(unsafe_code)]
//! #![feature(proc_macro)] // IMPORTANT always include this feature gate
//! #![no_std]
//! 
//! extern crate cortex_m_rtfm as rtfm; // IMPORTANT always do this rename
//! extern crate stm32f103xx; // the device crate
//! 
//! // import the procedural macro
//! use rtfm::app;
//! 
//! // This macro call indicates that this is a RTFM application
//! //
//! // This macro will expand to a `main` function so you don't need to supply
//! // `main` yourself.
//! app! {
//!     // this is a path to the device crate
//!     device: stm32f103xx,
//! }
//! 
//! // The initialization phase.
//! //
//! // This runs first and within a *global* critical section. Nothing can preempt
//! // this function.
//! fn init(p: init::Peripherals) {
//!     // This function has access to all the peripherals of the device
//!     p.GPIOA;
//!     p.RCC;
//!     // ..
//! 
//!     // You'll hit this breakpoint first
//!     rtfm::bkpt();
//! }
//! 
//! // The idle loop.
//! //
//! // This runs afterwards and has a priority of 0. All tasks can preempt this
//! // function. This function can never return so it must contain some sort of
//! // endless loop.
//! fn idle() -> ! {
//!     // And then this breakpoint
//!     rtfm::bkpt();
//! 
//!     loop {
//!         // This puts the processor to sleep until there's a task to service
//!         rtfm::wfi();
//!     }
//! }
//! ```
// Auto-generated. Do not modify.
