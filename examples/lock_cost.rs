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
        (Shared { shared: 0 }, Local {}, init::Monotonics())
    }

    #[idle(shared = [shared])]
    #[inline(never)]
    fn idle(mut cx: idle::Context) -> ! {
        cx.shared.shared.lock(|shared| *shared += 1);

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = UART0, shared = [shared])]
    fn uart0(mut cx: uart0::Context) {
        cx.shared.shared.lock(|shared| *shared += 1);
    }
}

// cargo objdump --example lock_cost --target thumbv7m-none-eabi --release --features inline-asm -- --disassemble > lock_cost.objdump

// 0000016c <lock_cost::app::idle::he8e6b27e7333515d>:
//      16c: 80 b5        	push	{r7, lr}
//      16e: 6f 46        	mov	r7, sp
//      170: 01 78        	ldrb	r1, [r0]
//      172: 39 b1        	cbz	r1, 0x184 <lock_cost::app::idle::he8e6b27e7333515d+0x18> @ imm = #14
//      174: 40 f2 00 00  	movw	r0, #0
//      178: c2 f2 00 00  	movt	r0, #8192
//      17c: 01 68        	ldr	r1, [r0]
//      17e: 01 31        	adds	r1, #1
//      180: 01 60        	str	r1, [r0]
//      182: 0f e0        	b	0x1a4 <lock_cost::app::idle::he8e6b27e7333515d+0x38> @ imm = #30
//      184: 01 21        	movs	r1, #1
//      186: 01 70        	strb	r1, [r0]
//      188: e0 21        	movs	r1, #224
//      18a: 81 f3 11 88  	msr	basepri, r1
//      18e: 40 f2 00 01  	movw	r1, #0
//      192: c2 f2 00 01  	movt	r1, #8192
//      196: 0a 68        	ldr	r2, [r1]
//      198: 01 32        	adds	r2, #1
//      19a: 0a 60        	str	r2, [r1]
//      19c: 00 21        	movs	r1, #0
//      19e: 81 f3 11 88  	msr	basepri, r1
//      1a2: 01 70        	strb	r1, [r0]
//      1a4: 00 bf        	nop
//      1a6: fd e7        	b	0x1a4 <lock_cost::app::idle::he8e6b27e7333515d+0x38> @ imm = #-6

// 000001a8 <UART0>:
//      1a8: 80 b5        	push	{r7, lr}
//      1aa: 6f 46        	mov	r7, sp
//      1ac: 40 f2 00 00  	movw	r0, #0
//      1b0: c2 f2 00 00  	movt	r0, #8192
//      1b4: 01 68        	ldr	r1, [r0]
//      1b6: 01 31        	adds	r1, #1
//      1b8: 01 60        	str	r1, [r0]
//      1ba: 00 20        	movs	r0, #0
//      1bc: 80 f3 11 88  	msr	basepri, r0
//      1c0: 80 bd        	pop	{r7, pc}
