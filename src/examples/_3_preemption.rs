//! Two tasks running at *different* priorities with access to the same resource
//!
//! ```
//! #![deny(unsafe_code)]
//! #![deny(warnings)]
//! #![no_std]
//! #![no_main]
//! 
//! #[macro_use(entry)]
//! extern crate cortex_m_rt as rt;
//! extern crate cortex_m_rtfm as rtfm;
//! extern crate panic_halt;
//! extern crate stm32f103xx;
//! 
//! use rtfm::{app, Resource, Threshold};
//! 
//! app! {
//!     device: stm32f103xx,
//! 
//!     resources: {
//!         static COUNTER: u64 = 0;
//!     },
//! 
//!     tasks: {
//!         // The `SysTick` task has higher priority than `TIM2`
//!         SysTick: {
//!             path: sys_tick,
//!             priority: 2,
//!             resources: [COUNTER],
//!         },
//! 
//!         TIM2: {
//!             path: tim2,
//!             priority: 1,
//!             resources: [COUNTER],
//!         },
//!     },
//! }
//! 
//! fn init(_p: init::Peripherals, _r: init::Resources) {
//!     // ..
//! }
//! 
//! fn idle() -> ! {
//!     loop {
//!         rtfm::wfi();
//!     }
//! }
//! 
//! fn sys_tick(_t: &mut Threshold, mut r: SysTick::Resources) {
//!     // ..
//! 
//!     // This task can't be preempted by `tim2` so it has direct access to the
//!     // resource data
//!     *r.COUNTER += 1;
//! 
//!     // ..
//! }
//! 
//! fn tim2(t: &mut Threshold, mut r: TIM2::Resources) {
//!     // ..
//! 
//!     // As this task runs at lower priority it needs a critical section to
//!     // prevent `sys_tick` from preempting it while it modifies this resource
//!     // data. The critical section is required to prevent data races which can
//!     // lead to undefined behavior.
//!     r.COUNTER.claim_mut(t, |counter, _t| {
//!         // `claim_mut` creates a critical section
//!         *counter += 1;
//!     });
//! 
//!     // ..
//! }
//! ```
// Auto-generated. Do not modify.
