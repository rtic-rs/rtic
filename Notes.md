# Notes for lock optimization

## Idea

Current implmentation always reads and writes BASEPRI on entry/exit of an interrupt (this is done by the `cortex-m-rtfm/src/export::run` which is a trampoline to execute the actual task).

Using this approch, we are reading BASEPRI if and only if we are actually changing BASEPRI.

On restoring BASEPRI (in `lock`) we chose to restore the original BASEPRI value if we at the outmost nesting level (initial priority of the task). In this way, we can avoid unnecessary BASEPRI accesses, and reduce register pressure.

If you want to play around checkout the `lockopt` branch and use:

``` shell
> arm-none-eabi-objdump target/thumbv7m-none-eabi/release/examples/lockopt -d > lockopt.asm
```

We extend `cortex-m-rtfm/src/export::Priority` with additional fields to store `init_logic` (priority of the task) and `old_basepri_hw`. The latter field is initially `None` on creation.

``` Rust
// Newtype over `Cell` that forbids mutation through a shared reference
pub struct Priority {
    init_logic: u8,
    current_logic: Cell<u8>,
    #[cfg(armv7m)]
    old_basepri_hw: Cell<Option<u8>>,
}

impl Priority {
    #[inline(always)]
    pub unsafe fn new(value: u8) -> Self {
        Priority {
            init_logic: value,
            current_logic: Cell::new(value),
            old_basepri_hw: Cell::new(None),
        }
    }

    #[inline(always)]
    fn set_logic(&self, value: u8) {
        self.current_logic.set(value)
    }

    #[inline(always)]
    fn get_logic(&self) -> u8 {
        self.current_logic.get()
    }

    #[inline(always)]
    fn get_init_logic(&self) -> u8 {
        self.init_logic
    }

    #[cfg(armv7m)]
    #[inline(always)]
    fn get_old_basepri_hw(&self) -> Option<u8> {
        self.old_basepri_hw.get()
    }

    #[cfg(armv7m)]
    #[inline(always)]
    fn set_old_basepri_hw(&self, value: u8) {
        self.old_basepri_hw.set(Some(value));
    }
}
```

The corresponding `lock` is implemented as follows:

``` Rust
#[cfg(armv7m)]
#[inline(always)]
pub unsafe fn lock<T, R>(
    ptr: *mut T,
    priority: &Priority,
    ceiling: u8,
    nvic_prio_bits: u8,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    let current = priority.get_logic();

    if current < ceiling {
        if ceiling == (1 << nvic_prio_bits) {
            priority.set_logic(u8::max_value());
            let r = interrupt::free(|_| f(&mut *ptr));
            priority.set_logic(current);
            r
        } else {
            match priority.get_old_basepri_hw() {
                None => priority.set_old_basepri_hw(basepri::read()),
                _ => (),
            };
            priority.set_logic(ceiling);
            basepri::write(logical2hw(ceiling, nvic_prio_bits));
            let r = f(&mut *ptr);
            if current == priority.get_init_logic() {
                basepri::write(priority.get_old_basepri_hw().unwrap());
            } else {
                basepri::write(logical2hw(priority.get_logic(), nvic_prio_bits));
            }
            priority.set_logic(current);
            r
        }
    } else {
        f(&mut *ptr)
    }
}
```

The highest priority is achieved through an `interrupt_free` and does not at all affect the `BASEPRI`. Thus it manipulates only the "logic" priority (used to optimize out locks).

For the normal case, on enter we check if the BASEPRI register has been read, if not we read it and update `old_basepri_hw`. On exit we check if we should restore a logical priority (inside a nested lock) or to restore the BASEPRI (previously stored in `old_basepri_hw`).  

## Safety

We can safely `unwrap` the `get_old_basepri_hw: Option<u8>` as the path leading up to the `unwrap` passes an update to `Some` or was already `Some`. Updating `get_old_basepri_hw` is monotonic, the API offers no way of making `get_old_basepri_hw` into `None` (besides `new`).

Moreover `new` is the only public function of `Priority`, thus we are exposing nothing dangerous to the user. (Externally changing `old_basepri_hw` could lead to memory unsafety, as an incorrect BASEPRI value may allow starting a task that should have been blocked, and once started access to resources with the same ceiling (or lower) is directly granted under SRP).

## Implementation

Implementation mainly regards two files, the `rtfm/src/export.rs` (discussed above) and `macros/src/codegen/hardware_tasks.rs`. For the latter the task dispatcher is updated as follows:

``` Rust
        ...
        const_app.push(quote!(
            #[allow(non_snake_case)]
            #[no_mangle]
            #section
            #cfg_core
            unsafe fn #symbol() {
                const PRIORITY: u8 = #priority;
                #let_instant
                crate::#name(
                    #locals_new
                    #name::Context::new(&rtfm::export::Priority::new(PRIORITY) #instant)
                    );
            }
        ));
        ...
```

Basically we create `Priority` (on stack) and use that to create a `Context`. The beauty is that LLVM is completely optimizing out the data structure (and related code), but taking into account its implications to control flow. Thus, the locks AND initial reading of BASEPRI will be optimized at compile time at Zero cost.

Overall, using this approach, we don't need a trampoline (`run`). We reduce the overhead by at least two machine instructions (additional reading/writing of BASEPRI) for each interrupt. It also reduces the register pressure (as less information needs to be stored).

## Evaluation

The `examples/lockopt.rs` shows that locks are effectively optimized out.

Old Implementation
``` asm
00000130 <GPIOB>:
 130:	21a0      	movs	r1, #160	; 0xa0
 132:	f3ef 8011 	mrs	r0, BASEPRI
 136:	f381 8811 	msr	BASEPRI, r1
 13a:	f240 0100 	movw	r1, #0
 13e:	f2c2 0100 	movt	r1, #8192	; 0x2000
 142:	680a      	ldr	r2, [r1, #0]
 144:	3201      	adds	r2, #1
 146:	600a      	str	r2, [r1, #0]
 148:	21c0      	movs	r1, #192	; 0xc0
 14a:	f381 8811 	msr	BASEPRI, r1
 14e:	f380 8811 	msr	BASEPRI, r0
 152:	4770      	bx	lr

00000154 <GPIOC>:
 154:	f240 0100 	movw	r1, #0
 158:	f3ef 8011 	mrs	r0, BASEPRI
 15c:	f2c2 0100 	movt	r1, #8192	; 0x2000
 160:	680a      	ldr	r2, [r1, #0]
 162:	3202      	adds	r2, #2
 164:	600a      	str	r2, [r1, #0]
 166:	f380 8811 	msr	BASEPRI, r0
 16a:	4770      	bx	lr
```

With lock opt. We see a 20% improvement for short/small tasks. 
``` asm
00000128 <GPIOB>:
 128:	21a0      	movs	r1, #160	; 0xa0
 12a:	f3ef 8011 	mrs	r0, BASEPRI
 12e:	f381 8811 	msr	BASEPRI, r1
 132:	f240 0100 	movw	r1, #0
 136:	f2c2 0100 	movt	r1, #8192	; 0x2000
 13a:	680a      	ldr	r2, [r1, #0]
 13c:	3201      	adds	r2, #1
 13e:	600a      	str	r2, [r1, #0]
 140:	f380 8811 	msr	BASEPRI, r0
 144:	4770      	bx	lr

00000146 <GPIOC>:
 146:	f240 0000 	movw	r0, #0
 14a:	f2c2 0000 	movt	r0, #8192	; 0x2000
 14e:	6801      	ldr	r1, [r0, #0]
 150:	3102      	adds	r1, #2
 152:	6001      	str	r1, [r0, #0]
 154:	4770      	bx	lr
```

GPIOB/C are sharing a resource (C higher prio). Notice, for GPIOC there is no BASEPRI manipulation at all.

For GPIOB, there is a single read of BASEPRI (stored in `old_basepri_hw`) and just two writes, one for entering critical section, one for exiting. On exit we detect that we are indeed at the initial priority for the task, thus we restore the `old_basepri_hw` instead of a logic priority.

## Limitations and Drawbacks

None spotted so far.

## Observations

``` shell
> llvm-objdump target/thumbv7m-none-eabi/release/examples/lockopt -d > lockopt.asm

> cargo objdump --example lockopt --release -- -d > lockopt.asm
```

Neither give assembly dump with symbols (very annoying to rely on `arm-none-eabi-objdump` for proper objdumps), maybe just an option is missing?
