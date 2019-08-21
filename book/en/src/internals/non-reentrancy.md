# Non-reentrancy

In RTFM, tasks handlers are *not* reentrant. Reentering a task handler can break
Rust aliasing rules and lead to *undefined behavior*. A task handler can be
reentered in one of two ways: in software or by hardware.

## In software

To reenter a task handler in software its underlying interrupt handler must be
invoked using FFI (see example below). FFI requires `unsafe` code so end users
are discouraged from directly invoking an interrupt handler.

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    #[init]
    fn init(c: init::Context) { .. }

    #[interrupt(binds = UART0)]
    fn foo(c: foo::Context) {
        static mut X: u64 = 0;

        let x: &mut u64 = X;

        // ..

        //~ `bar` can preempt `foo` at this point

        // ..
    }

    #[interrupt(binds = UART1, priority = 2)]
    fn bar(c: foo::Context) {
        extern "C" {
            fn UART0();
        }

        // this interrupt handler will invoke task handler `foo` resulting
        // in aliasing of the static variable `X`
        unsafe { UART0() }
    }
};
```

The RTFM framework must generate the interrupt handler code that calls the user
defined task handlers. We are careful in making these handlers impossible to
call from user code.

The above example expands into:

``` rust
fn foo(c: foo::Context) {
    // .. user code ..
}

fn bar(c: bar::Context) {
    // .. user code ..
}

const APP: () = {
    // everything in this block is not visible to user code

    #[no_mangle]
    unsafe fn USART0() {
        foo(..);
    }

    #[no_mangle]
    unsafe fn USART1() {
        bar(..);
    }
};
```

## By hardware

A task handler can also be reentered without software intervention. This can
occur if the same handler is assigned to two or more interrupts in the vector
table but there's no syntax for this kind of configuration in the RTFM
framework.
