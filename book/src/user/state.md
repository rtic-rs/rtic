# Adding state

Tasks are stateless by default; state can be added by assigning them *resources*. Resources are
`static` variables that can be assigned to tasks. If a resource is assigned to a single task then
it's *owned* by that task and the task has exclusive access to the resource. A resource can also be
*shared* by two or more tasks; when shared a resource must be `claim`ed (which may involve a lock)
before its data can be accessed -- this prevents data races. In RTFM it's preferred to use message
passing (more on that later) instead of sharing state.

The following example shows how to assign a resource to a task to preserve state across the
different invocations of the task.

``` rust
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm;

use cortex_m_rtfm::app;

app! {
    device: stm32f103xx,

    // declare resources
    resources: {
        // number of times the user pressed the button
        static PRESSES: u32 = 0;
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,

            // assign the `PRESSES` resource to the `exti0` task
            resources: [PRESSES],
        },
    },
}

// omitted: `init` and `idle`

fn exti0(ctxt: exti0::Context) {
    let presses: &mut u32 = ctxt.resources.PRESSES;
    *presses += 1;

    println!("Button pressed {} times!", *presses);
}
```
