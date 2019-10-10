# Critical sections

When a resource (static variable) is shared between two, or more, tasks that run
at different priorities some form of mutual exclusion is required to mutate the
memory in a data race free manner. In RTFM we use priority-based critical
sections to guarantee mutual exclusion (see the [Immediate Ceiling Priority
Protocol][icpp]).

[icpp]: https://en.wikipedia.org/wiki/Priority_ceiling_protocol

The critical section consists of temporarily raising the *dynamic* priority of
the task. While a task is within this critical section all the other tasks that
may request the resource are *not allowed to start*.

How high must the dynamic priority be to ensure mutual exclusion on a particular
resource? The [ceiling analysis](ceilings.html) is in charge of
answering that question and will be discussed in the next section. This section
will focus on the implementation of the critical section.

## Resource proxy

For simplicity, let's look at a resource shared by two tasks that run at
different priorities. Clearly one of the task can preempt the other; to prevent
a data race the *lower priority* task must use a critical section when it needs
to modify the shared memory. On the other hand, the higher priority task can
directly modify the shared memory because it can't be preempted by the lower
priority task. To enforce the use of a critical section on the lower priority
task we give it a *resource proxy*, whereas we give a unique reference
(`&mut-`) to the higher priority task.

The example below shows the different types handed out to each task:

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    struct Resources {
        #[init(0)]
        x: u64,
    }

    #[interrupt(binds = UART0, priority = 1, resources = [x])]
    fn foo(c: foo::Context) {
        // resource proxy
        let mut x: resources::x = c.resources.x;

        x.lock(|x: &mut u64| {
            // critical section
            *x += 1
        });
    }

    #[interrupt(binds = UART1, priority = 2, resources = [x])]
    fn bar(c: bar::Context) {
        let mut x: &mut u64 = c.resources.x;

        *x += 1;
    }

    // ..
};
```

Now let's see how these types are created by the framework.

``` rust
fn foo(c: foo::Context) {
    // .. user code ..
}

fn bar(c: bar::Context) {
    // .. user code ..
}

pub mod resources {
    pub struct x {
        // ..
    }
}

pub mod foo {
    pub struct Resources {
        pub x: resources::x,
    }

    pub struct Context {
        pub resources: Resources,
        // ..
    }
}

pub mod bar {
    pub struct Resources<'a> {
        pub x: &'a mut u64,
    }

    pub struct Context {
        pub resources: Resources,
        // ..
    }
}

const APP: () = {
    static mut x: u64 = 0;

    impl rtfm::Mutex for resources::x {
        type T = u64;

        fn lock<R>(&mut self, f: impl FnOnce(&mut u64) -> R) -> R {
            // we'll check this in detail later
        }
    }

    #[no_mangle]
    unsafe fn UART0() {
        foo(foo::Context {
            resources: foo::Resources {
                x: resources::x::new(/* .. */),
            },
            // ..
        })
    }

    #[no_mangle]
    unsafe fn UART1() {
        bar(bar::Context {
            resources: bar::Resources {
                x: &mut x,
            },
            // ..
        })
    }
};
```

## `lock`

Let's now zoom into the critical section itself. In this example, we have to
raise the dynamic priority to at least `2` to prevent a data race. On the
Cortex-M architecture the dynamic priority can be changed by writing to the
`BASEPRI` register.

The semantics of the `BASEPRI` register are as follows:

- Writing a value of `0` to `BASEPRI` disables its functionality.
- Writing a non-zero value to `BASEPRI` changes the priority level required for
  interrupt preemption. However, this only has an effect when the written value
  is *lower* than the priority level of current execution context, but note that
  a lower hardware priority level means higher logical priority

Thus the dynamic priority at any point in time can be computed as

``` rust
dynamic_priority = max(hw2logical(BASEPRI), hw2logical(static_priority))
```

Where `static_priority` is the priority programmed in the NVIC for the current
interrupt, or a logical `0` when the current context is `idle`.

In this particular example we could implement the critical section as follows:

> **NOTE:** this is a simplified implementation

``` rust
impl rtfm::Mutex for resources::x {
    type T = u64;

    fn lock<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut u64) -> R,
    {
        unsafe {
            // start of critical section: raise dynamic priority to `2`
            asm!("msr BASEPRI, 192" : : : "memory" : "volatile");

            // run user code within the critical section
            let r = f(&mut x);

            // end of critical section: restore dynamic priority to its static value (`1`)
            asm!("msr BASEPRI, 0" : : : "memory" : "volatile");

            r
        }
    }
}
```

Here it's important to use the `"memory"` clobber in the `asm!` block. It
prevents the compiler from reordering memory operations across it. This is
important because accessing the variable `x` outside the critical section would
result in a data race.

It's important to note that the signature of the `lock` method prevents nesting
calls to it. This is required for memory safety, as nested calls would produce
multiple unique references (`&mut-`) to `x` breaking Rust aliasing rules. See
below:

``` rust
#[interrupt(binds = UART0, priority = 1, resources = [x])]
fn foo(c: foo::Context) {
    // resource proxy
    let mut res: resources::x = c.resources.x;

    res.lock(|x: &mut u64| {
        res.lock(|alias: &mut u64| {
            //~^ error: `res` has already been uniquely borrowed (`&mut-`)
            // ..
        });
    });
}
```

## Nesting

Nesting calls to `lock` on the *same* resource must be rejected by the compiler
for memory safety but nesting `lock` calls on *different* resources is a valid
operation. In that case we want to make sure that nesting critical sections
never results in lowering the dynamic priority, as that would be unsound, and we
also want to optimize the number of writes to the `BASEPRI` register and
compiler fences. To that end we'll track the dynamic priority of the task using
a stack variable and use that to decide whether to write to `BASEPRI` or not. In
practice, the stack variable will be optimized away by the compiler but it still
provides extra information to the compiler.

Consider this program:

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    struct Resources {
        #[init(0)]
        x: u64,
        #[init(0)]
        y: u64,
    }

    #[init]
    fn init() {
        rtfm::pend(Interrupt::UART0);
    }

    #[interrupt(binds = UART0, priority = 1, resources = [x, y])]
    fn foo(c: foo::Context) {
        let mut x = c.resources.x;
        let mut y = c.resources.y;

        y.lock(|y| {
            *y += 1;

            *x.lock(|x| {
                x += 1;
            });

            *y += 1;
        });

        // mid-point

        x.lock(|x| {
            *x += 1;

            y.lock(|y| {
                *y += 1;
            });

            *x += 1;
        })
    }

    #[interrupt(binds = UART1, priority = 2, resources = [x])]
    fn bar(c: foo::Context) {
        // ..
    }

    #[interrupt(binds = UART2, priority = 3, resources = [y])]
    fn baz(c: foo::Context) {
        // ..
    }

    // ..
};
```

The code generated by the framework looks like this:

``` rust
// omitted: user code

pub mod resources {
    pub struct x<'a> {
        priority: &'a Cell<u8>,
    }

    impl<'a> x<'a> {
        pub unsafe fn new(priority: &'a Cell<u8>) -> Self {
            x { priority }
        }

        pub unsafe fn priority(&self) -> &Cell<u8> {
            self.priority
        }
    }

    // repeat for `y`
}

pub mod foo {
    pub struct Context {
        pub resources: Resources,
        // ..
    }

    pub struct Resources<'a> {
        pub x: resources::x<'a>,
        pub y: resources::y<'a>,
    }
}

const APP: () = {
    use cortex_m::register::basepri;

    #[no_mangle]
    unsafe fn UART1() {
        // the static priority of this interrupt (as specified by the user)
        const PRIORITY: u8 = 2;

        // take a snashot of the BASEPRI
        let initial = basepri::read();

        let priority = Cell::new(PRIORITY);
        bar(bar::Context {
            resources: bar::Resources::new(&priority),
            // ..
        });

        // roll back the BASEPRI to the snapshot value we took before
        basepri::write(initial); // same as the `asm!` block we saw before
    }

    // similarly for `UART0` / `foo` and `UART2` / `baz`

    impl<'a> rtfm::Mutex for resources::x<'a> {
        type T = u64;

        fn lock<R>(&mut self, f: impl FnOnce(&mut u64) -> R) -> R {
            unsafe {
                // the priority ceiling of this resource
                const CEILING: u8 = 2;

                let current = self.priority().get();
                if current < CEILING {
                    // raise dynamic priority
                    self.priority().set(CEILING);
                    basepri::write(logical2hw(CEILING));

                    let r = f(&mut y);

                    // restore dynamic priority
                    basepri::write(logical2hw(current));
                    self.priority().set(current);

                    r
                } else {
                    // dynamic priority is high enough
                    f(&mut y)
                }
            }
        }
    }

    // repeat for resource `y`
};
```

At the end the compiler will optimize the function `foo` into something like
this:

``` rust
fn foo(c: foo::Context) {
    // NOTE: BASEPRI contains the value `0` (its reset value) at this point

    // raise dynamic priority to `3`
    unsafe { basepri::write(160) }

    // the two operations on `y` are merged into one
    y += 2;

    // BASEPRI is not modified to access `x` because the dynamic priority is high enough
    x += 1;

    // lower (restore) the dynamic priority to `1`
    unsafe { basepri::write(224) }

    // mid-point

    // raise dynamic priority to `2`
    unsafe { basepri::write(192) }

    x += 1;

    // raise dynamic priority to `3`
    unsafe { basepri::write(160) }

    y += 1;

    // lower (restore) the dynamic priority to `2`
    unsafe { basepri::write(192) }

    // NOTE: it would be sound to merge this operation on `x` with the previous one but
    // compiler fences are coarse grained and prevent such optimization
    x += 1;

    // lower (restore) the dynamic priority to `1`
    unsafe { basepri::write(224) }

    // NOTE: BASEPRI contains the value `224` at this point
    // the UART0 handler will restore the value to `0` before returning
}
```

## The BASEPRI invariant

An invariant that the RTFM framework has to preserve is that the value of the
BASEPRI at the start of an *interrupt* handler must be the same value it has
when the interrupt handler returns. BASEPRI may change during the execution of
the interrupt handler but running an interrupt handler from start to finish
should not result in an observable change of BASEPRI.

This invariant needs to be preserved to avoid raising the dynamic priority of a
handler through preemption. This is best observed in the following example:

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    struct Resources {
        #[init(0)]
        x: u64,
    }

    #[init]
    fn init() {
        // `foo` will run right after `init` returns
        rtfm::pend(Interrupt::UART0);
    }

    #[task(binds = UART0, priority = 1)]
    fn foo() {
        // BASEPRI is `0` at this point; the dynamic priority is currently `1`

        // `bar` will preempt `foo` at this point
        rtfm::pend(Interrupt::UART1);

        // BASEPRI is `192` at this point (due to a bug); the dynamic priority is now `2`
        // this function returns to `idle`
    }

    #[task(binds = UART1, priority = 2, resources = [x])]
    fn bar() {
        // BASEPRI is `0` (dynamic priority = 2)

        x.lock(|x| {
            // BASEPRI is raised to `160` (dynamic priority = 3)

            // ..
        });

        // BASEPRI is restored to `192` (dynamic priority = 2)
    }

    #[idle]
    fn idle() -> ! {
        // BASEPRI is `192` (due to a bug); dynamic priority = 2

        // this has no effect due to the BASEPRI value
        // the task `foo` will never be executed again
        rtfm::pend(Interrupt::UART0);

        loop {
            // ..
        }
    }

    #[task(binds = UART2, priority = 3, resources = [x])]
    fn baz() {
        // ..
    }

};
```

IMPORTANT: let's say we *forget* to roll back `BASEPRI` in `UART1` -- this would
be a bug in the RTFM code generator.

``` rust
// code generated by RTFM

const APP: () = {
    // ..

    #[no_mangle]
    unsafe fn UART1() {
        // the static priority of this interrupt (as specified by the user)
        const PRIORITY: u8 = 2;

        // take a snashot of the BASEPRI
        let initial = basepri::read();

        let priority = Cell::new(PRIORITY);
        bar(bar::Context {
            resources: bar::Resources::new(&priority),
            // ..
        });

        // BUG: FORGOT to roll back the BASEPRI to the snapshot value we took before
        basepri::write(initial);
    }
};
```

The consequence is that `idle` will run at a dynamic priority of `2` and in fact
the system will never again run at a dynamic priority lower than `2`. This
doesn't compromise the memory safety of the program but affects task scheduling:
in this particular case tasks with a priority of `1` will never get a chance to
run.
