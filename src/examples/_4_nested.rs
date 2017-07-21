//! Nesting claims and how the preemption threshold works
//!
//! If you run this program you'll hit the breakpoints as indicated by the
//! letters in the comments: A, then B, then C, etc.
//!
//! ```
//! 
//! #![deny(unsafe_code)]
//! #![feature(const_fn)]
//! #![feature(proc_macro)]
//! #![no_std]
//! 
//! #[macro_use(task)]
//! extern crate cortex_m_rtfm as rtfm;
//! extern crate stm32f103xx;
//! 
//! use stm32f103xx::Interrupt;
//! use rtfm::{app, Resource, Threshold};
//! 
//! app! {
//!     device: stm32f103xx,
//! 
//!     resources: {
//!         static LOW: u64 = 0;
//!         static HIGH: u64 = 0;
//!     },
//! 
//!     tasks: {
//!         EXTI0: {
//!             enabled: true,
//!             priority: 1,
//!             resources: [LOW, HIGH],
//!         },
//! 
//!         EXTI1: {
//!             enabled: true,
//!             priority: 2,
//!             resources: [LOW],
//!         },
//! 
//!         EXTI2: {
//!             enabled: true,
//!             priority: 3,
//!             resources: [HIGH],
//!         },
//!     },
//! }
//! 
//! fn init(_p: init::Peripherals, _r: init::Resources) {}
//! 
//! fn idle() -> ! {
//!     // sets task `exti0` as pending
//!     //
//!     // because `exti0` has higher priority than `idle` it will be executed
//!     // immediately
//!     rtfm::set_pending(Interrupt::EXTI0); // ~> exti0
//! 
//!     loop {
//!         rtfm::wfi();
//!     }
//! }
//! 
//! task!(EXTI0, exti0);
//! 
//! fn exti0(t: &mut Threshold, r: EXTI0::Resources) {
//!     // because this task has a priority of 1 the preemption threshold is also 1
//! 
//!     // A
//!     rtfm::bkpt();
//! 
//!     // because `exti1` has higher priority than `exti0` it can preempt it
//!     rtfm::set_pending(Interrupt::EXTI1); // ~> exti1
//! 
//!     // a claim creates a critical section
//!     r.LOW.claim_mut(t, |_low, t| {
//!         // this claim increases the preemption threshold to 2
//!         // just high enough to not race with task `exti1` for access to the
//!         // `LOW` resource
//! 
//!         // C
//!         rtfm::bkpt();
//! 
//!         // now `exti1` can't preempt this task because its priority is equal to
//!         // the current preemption threshold
//!         rtfm::set_pending(Interrupt::EXTI1);
//! 
//!         // but `exti2` can, because its priority is higher than the current
//!         // preemption threshold
//!         rtfm::set_pending(Interrupt::EXTI2); // ~> exti2
//! 
//!         // E
//!         rtfm::bkpt();
//! 
//!         // claims can be nested
//!         r.HIGH.claim_mut(t, |_high, _| {
//!             // This claim increases the preemption threshold to 3
//! 
//!             // now `exti2` can't preempt this task
//!             rtfm::set_pending(Interrupt::EXTI2);
//! 
//!             // F
//!             rtfm::bkpt();
//!         });
//! 
//!         // upon leaving the critical section the preemption threshold drops to 2
//!         // and `exti2` immediately preempts this task
//!         // ~> exti2
//!     });
//! 
//!     // once again the preemption threshold drops to 1
//!     // now the pending `exti1` can preempt this task
//!     // ~> exti1
//! }
//! 
//! task!(EXTI1, exti1);
//! 
//! fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {
//!     // B, H
//!     rtfm::bkpt();
//! }
//! 
//! task!(EXTI2, exti2);
//! 
//! fn exti2(_t: &mut Threshold, _r: EXTI2::Resources) {
//!     // D, G
//!     rtfm::bkpt();
//! }
//! ```
// Auto-generated. Do not modify.
