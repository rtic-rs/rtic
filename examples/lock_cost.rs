//! examples/lock_cost.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[shared]
    struct Shared {
        shared: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        use cortex_m_semihosting::debug;
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        (Shared { shared: 0 }, Local {}, init::Monotonics())
    }

    #[task(binds = GPIOA, shared = [shared])]
    fn low(mut cx: low::Context) {
        cx.shared.shared.lock(|shared| *shared += 1);
    }

    #[task(binds = GPIOB, priority = 2, shared = [shared])]
    fn high(mut cx: high::Context) {
        cx.shared.shared.lock(|shared| *shared += 1);
    }
}

// cargo objdump --example lock_cost --target thumbv7m-none-eabi --release --features inline-asm -- --disassemble > lock_cost.objdump
//
// Zero-Cost implementations:
// 0000016c <GPIOA>:
//      16c: 80 b5        	push	{r7, lr}
//      16e: 6f 46        	mov	r7, sp
//      170: c0 20        	movs	r0, #192
//      172: 80 f3 11 88  	msr	basepri, r0
//      176: 40 f2 00 00  	movw	r0, #0
//      17a: c2 f2 00 00  	movt	r0, #8192
//      17e: 01 68        	ldr	r1, [r0]
//      180: 01 31        	adds	r1, #1
//      182: 01 60        	str	r1, [r0]
//      184: e0 20        	movs	r0, #224
//      186: 80 f3 11 88  	msr	basepri, r0
//      18a: 00 20        	movs	r0, #0
//      18c: 80 f3 11 88  	msr	basepri, r0
//      190: 80 bd        	pop	{r7, pc}

// 00000192 <GPIOB>:
//      192: 80 b5        	push	{r7, lr}
//      194: 6f 46        	mov	r7, sp
//      196: 40 f2 00 01  	movw	r1, #0
//      19a: ef f3 11 80  	mrs	r0, basepri
//      19e: c2 f2 00 01  	movt	r1, #8192
//      1a2: 0a 68        	ldr	r2, [r1]
//      1a4: 01 32        	adds	r2, #1
//      1a6: 0a 60        	str	r2, [r1]
//      1a8: 80 f3 11 88  	msr	basepri, r0
//      1ac: 80 bd        	pop	{r7, pc}
