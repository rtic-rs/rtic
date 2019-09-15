# Ceiling analysis

A resource *priority ceiling*, or just *ceiling*, is the dynamic priority that
any task must have to safely access the resource memory. Ceiling analysis is
relatively simple but critical to the memory safety of RTFM applications.

To compute the ceiling of a resource we must first collect a list of tasks that
have access to the resource -- as the RTFM framework enforces access control to
resources at compile time it also has access to this information at compile
time. The ceiling of the resource is simply the highest logical priority among
those tasks.

`init` and `idle` are not proper tasks but they can access resources so they
need to be considered in the ceiling analysis. `idle` is considered as a task
that has a logical priority of `0` whereas `init` is completely omitted from the
analysis -- the reason for that is that `init` never uses (or needs) critical
sections to access static variables.

In the previous section we showed that a shared resource may appear as a unique
reference (`&mut-`) or behind a proxy depending on the task that has access to
it. Which version is presented to the task depends on the task priority and the
resource ceiling. If the task priority is the same as the resource ceiling then
the task gets a unique reference (`&mut-`) to the resource memory, otherwise the
task gets a proxy -- this also applies to `idle`. `init` is special: it always
gets a unique reference (`&mut-`) to resources.

An example to illustrate the ceiling analysis:

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    struct Resources {
        // accessed by `foo` (prio = 1) and `bar` (prio = 2)
        // -> CEILING = 2
        #[init(0)]
        x: u64,

        // accessed by `idle` (prio = 0)
        // -> CEILING = 0
        #[init(0)]
        y: u64,
    }

    #[init(resources = [x])]
    fn init(c: init::Context) {
        // unique reference because this is `init`
        let x: &mut u64 = c.resources.x;

        // unique reference because this is `init`
        let y: &mut u64 = c.resources.y;

        // ..
    }

    // PRIORITY = 0
    #[idle(resources = [y])]
    fn idle(c: idle::Context) -> ! {
        // unique reference because priority (0) == resource ceiling (0)
        let y: &'static mut u64 = c.resources.y;

        loop {
            // ..
        }
    }

    #[interrupt(binds = UART0, priority = 1, resources = [x])]
    fn foo(c: foo::Context) {
        // resource proxy because task priority (1) < resource ceiling (2)
        let x: resources::x = c.resources.x;

        // ..
    }

    #[interrupt(binds = UART1, priority = 2, resources = [x])]
    fn bar(c: foo::Context) {
        // unique reference because task priority (2) == resource ceiling (2)
        let x: &mut u64 = c.resources.x;

        // ..
    }

    // ..
};
```
