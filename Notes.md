# Notes for lock optimization

## Idea

We are reading BASEPRI independently if and only if we are actually changing BASEPRI.
On restoring BASEPRI chose to restore read value if at the outmost nesting level (initial priority of the task). In this way, unnecessary BASEPRI accesses, and reduce register pressure. 

If you want to play around checkout the `lockopt` branch and use:

``` shell
> arm-none-eabi-objdump target/thumbv7m-none-eabi/release/examples/lockopt -d > lockopt.asm
```

Extend `cortex-m-rtfm/src/export::Priority` with an additional fields to store `init_logic` (priority of the task) and `old_basepri_hw`. The latter field is initially `None` on creation.

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

The highest priority is achieved through an `interrupt_free` and does not at all affect the `BASEPRI`.

For the normal case, on enter we check if the BASEPRI register has been read, if not we read it and update `priority`. On exit we check if are to restore a logical priority (inside a nested lock) or to restore the BASEPRI (previously read).  

## Safety

We can safely `unwrap` the `get_old_basepri_hw: Option<u8>` as the path leading up to the `unwrap` passes an update to `Some` or was already `Some`. Updating `get_old_basepri_hw` is monotonic, the API offers no way of making `get_old_basepri_hw` into `None` (besides `new`).

Moreover `new` is the only public function of `Priority`, thus we are exposing nothing dangerous to the user.

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

``` asm
00000132 <GPIOB>:
 132:	b510      	push	{r4, lr}
 134:	f000 f893 	bl	25e <__basepri_r>
 138:	4604      	mov	r4, r0
 13a:	20a0      	movs	r0, #160	; 0xa0
 13c:	f000 f892 	bl	264 <__basepri_w>
 140:	f240 0000 	movw	r0, #0
 144:	f2c2 0000 	movt	r0, #8192	; 0x2000
 148:	6801      	ldr	r1, [r0, #0]
 14a:	3101      	adds	r1, #1
 14c:	6001      	str	r1, [r0, #0]
 14e:	4620      	mov	r0, r4
 150:	e8bd 4010 	ldmia.w	sp!, {r4, lr}
 154:	f000 b886 	b.w	264 <__basepri_w>

00000158 <GPIOC>:
 158:	f240 0000 	movw	r0, #0
 15c:	f2c2 0000 	movt	r0, #8192	; 0x2000
 160:	6801      	ldr	r1, [r0, #0]
 162:	3102      	adds	r1, #2
 164:	6001      	str	r1, [r0, #0]
 166:	4770      	bx	lr
```

GPIOB/C are sharing a resource (C higher prio). Notice, there is no BASEPRI manipulation at all.

For GPIOB, there is a single read of BASEPRI (stored in `old_basepri_hw`) and just two writes, one for entering critical section, one for exiting. On exit we detect that we are indeed at the initial priority for the task, thus we restore the `old_basepri_hw` instead of a logic priority.

## Limitations and Drawbacks

None spotted so far.

## Observations

``` shell
> llvm-objdump target/thumbv7m-none-eabi/release/examples/lockopt -d > lockopt.asm

> cargo objdump --example lockopt --release -- -d > lockopt.asm
```

Neither give assembly dump with symbols (very annoying to rely on `arm-none-eabi-objdump` for proper objdumps), maybe just an option is missing?
