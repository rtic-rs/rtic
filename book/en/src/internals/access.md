# Access control

One of the core foundations of RTFM is access control. Controlling which parts
of the program can access which static variables is instrumental to enforcing
memory safety.

Static variables are used to share state between interrupt handlers, or between
interrupts handlers and the bottom execution context, `main`. In normal Rust
code it's hard to have fine grained control over which functions can access a
static variable because static variables can be accessed from any function that
resides in the same scope in which they are declared. Modules give some control
over how a static variable can be accessed by they are not flexible enough.

To achieve the fine-grained access control where tasks can only access the
static variables (resources) that they have specified in their RTFM attribute
the RTFM framework performs a source code level transformation. This
transformation consists of placing the resources (static variables) specified by
the user *inside* a `const` item and the user code *outside* the `const` item.
This makes it impossible for the user code to refer to these static variables.

Access to the resources is then given to each task using a `Resources` struct
whose fields correspond to the resources the task has access to. There's one
such struct per task and the `Resources` struct is initialized with either a
unique reference (`&mut-`) to the static variables or with a resource proxy (see
section on [critical sections](critical-sections.html)).

The code below is an example of the kind of source level transformation that
happens behind the scenes:

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    static mut X: u64: 0;
    static mut Y: bool: 0;

    #[init(resources = [Y])]
    fn init(c: init::Context) {
        // .. user code ..
    }

    #[interrupt(binds = UART0, resources = [X])]
    fn foo(c: foo::Context) {
        // .. user code ..
    }

    #[interrupt(binds = UART1, resources = [X, Y])]
    fn bar(c: bar::Context) {
        // .. user code ..
    }

    // ..
};
```

The framework produces codes like this:

``` rust
fn init(c: init::Context) {
    // .. user code ..
}

fn foo(c: foo::Context) {
    // .. user code ..
}

fn bar(c: bar::Context) {
    // .. user code ..
}

// Public API
pub mod init {
    pub struct Context<'a> {
        pub resources: Resources<'a>,
        // ..
    }

    pub struct Resources<'a> {
        pub Y: &'a mut bool,
    }
}

pub mod foo {
    pub struct Context<'a> {
        pub resources: Resources<'a>,
        // ..
    }

    pub struct Resources<'a> {
        pub X: &'a mut u64,
    }
}

pub mod bar {
    pub struct Context<'a> {
        pub resources: Resources<'a>,
        // ..
    }

    pub struct Resources<'a> {
        pub X: &'a mut u64,
        pub Y: &'a mut bool,
    }
}

/// Implementation details
const APP: () = {
    // everything inside this `const` item is hidden from user code

    static mut X: u64 = 0;
    static mut Y: bool = 0;

    // the real entry point of the program
    unsafe fn main() -> ! {
        interrupt::disable();

        // ..

        // call into user code; pass references to the static variables
        init(init::Context {
            resources: init::Resources {
                X: &mut X,
            },
            // ..
        });

        // ..

        interrupt::enable();

        // ..
    }

    // interrupt handler that `foo` binds to
    #[no_mangle]
    unsafe fn UART0() {
        // call into user code; pass references to the static variables
        foo(foo::Context {
            resources: foo::Resources {
                X: &mut X,
            },
            // ..
        });
    }

    // interrupt handler that `bar` binds to
    #[no_mangle]
    unsafe fn UART1() {
        // call into user code; pass references to the static variables
        bar(bar::Context {
            resources: bar::Resources {
                X: &mut X,
                Y: &mut Y,
            },
            // ..
        });
    }
};
```
