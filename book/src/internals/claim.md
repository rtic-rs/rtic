# `claim`

At the center of RTFM we have the `Resource` abstraction. A `Resource` is a mechanism to share data
between two or more tasks (contexts of execution) that can potentially run at different priorities.
When tasks have different priorities they can preempt each other and this can lead to data races if
the access to the data is *not* synchronized. A `Resource` eliminates the data race problem by
forcing the tasks to access the data through a critical section. While in a critical section the
other tasks that share the `Resource` can *not* start.

As tasks in RTFM are all dispatched in interrupt handlers one way to create a critical section is to
disable all interrupts (`cpsid i` instruction). However, this approach also prevents tasks that are
not contending for the resource from starting, which can reduce the responsiveness of the system.
The Cortex-M implementation uses priority based critical sections (AKA Priority Ceiling Protocol) to
avoid this problem, or at least to reduce its effect.

The NVIC, which is the core of the RTFM scheduler, supports dynamic reprioritization of interrupts
via the [BASEPRI] register. By writing to this register we can increase the priority of the current
interrupt / task preventing tasks with lower priority from starting. A temporal increase of the
priority can be used as a critical section; this is how `claim` works in the Cortex-M implementation
of RTFM.

[BASEPRI]: https://developer.arm.com/products/architecture/m-profile/docs/100701/latest/special-purpose-mask-registers

The question is how much to increase the priority in these critical sections? The value must be high
enough to prevent data races but not too high that it blocks unrelated tasks. The answer to this
question comes from the Priority Ceiling Protocol: each resource has a priority *ceiling*; to access
the data a critical section must be created by temporarily increasing the priority to match the
priority ceiling; the priority ceiling of a resource is equal to the priority of the highest
priority task that can access the resource.

In the Cortex-M implementation of RTFM we store the ceiling of a resource in the type system and we
also track the dynamic priority of a task using the type system. The main reason for this is
generating optimal machine code for `claim`s.

Here's what the `Resource` abstraction looks like:

``` rust
/// Priority token
pub struct Priority<P> { _not_send_or_sync: *const (), _priority: PhantomData<P> }

pub unsafe trait Resource {
    /// The number of priority bits supported by the NVIC (device specific)
    const NVIC_PRIO_BITS: u8;

    /// The priority "ceiling" of this resource
    type Ceiling: Unsigned; // type level integer (cf. typenum)

    /// The data protected by this resource
    type Data: 'static + Send;

    // Returns a reference to the `static mut` variable protected by this resource
    #[doc(hidden)]
    unsafe fn _var() -> &'static mut Self::Data;

    /// Borrows the resource data while the priority is high enough
    // NOTE there's a mutable version of this method: `borrow_mut`
    fn borrow<P, 'p>(&'t self, p: &'p Priority<P>) -> &'p Self::Data
    where
        P: IsGreaterOrEqual<Self::Ceiling, Output = True>,
    {
        unsafe { Self::_var() }
    }

    /// Claim the data proceted by this resource
    // NOTE there's a mutable version of this method: `claim_mut`
    fn claim<P>(&self, t: &mut Priority<P>, f: F)
    where
        F: FnOnce(&Self::Data, &mut Priority<Maximum<P, Self::Ceiling>)
        P: Max<Self::Ceiling> + Unsigned,
        Self::Ceiling: Unsigned,
    {
        unsafe {
            if P::to_u8() >= Self::Ceiling::to_u8() {
                // the priority doesn't need to be raised further
                f(Self::get(), &mut Priority::new())
            } else {
                // the hardware priority ceiling of this resource
                let new = (1 << Self::NVIC_PRIO_BITS - Self::Ceiling::to_u8()) <<
                    (8 - Self::NVIC_PRIO_BITS);

                let old = basepri::read();

                // start the critical section by raising the dynamic priority
                basepri::write(new);

                // execute user provided code inside the critical section
                let r = f(Self::get(), &mut Priority::new());

                // end the critical section by restoring the old dynamic priority
                basepri::write(old);

                r
            }
        }
    }
}
```

The `Priority` *token* is used to track the current dynamic priority of a task. When a task starts
its `Context` contains a `Priority` token that represents the priority declared in `app!`. For
example, if the task priority was set to `2` the threshold token will have type `Threshold<U2>`
where `U2` is the type level version of `2` (cf. [`typenum`]).

[`typenum`]: https://docs.rs/typenum

The `claim` method creates a critical section by temporarily raising the task priority. Within this
critical section (closure) a new `Priority` token is provided while the outer `Priority` token is
invalidated due to borrow semantics (mutably borrowed / frozen).

When generating code the `app!` macro creates a `struct` that implements the `Resource` trait for
each resource declared in `resources`. The data behind each `Resource` is a `static mut` variable:

``` rust
// given: `resources: { static FOO: u32 = 0 }`

// app! produces
mod __resource {
    pub struct FOO { _not_send_or_sync: *const () }

    unsafe impl Resource for FOO {
        const NVIC_PRIO_BITS = stm32f103xx::NVIC_PRIO_BITS;

        type Ceiling = U3;

        type Data = u32;

        unsafe fn _var() -> &'static mut u32 {
            static mut FOO: u32 = 0;

            &mut FOO
        }
    }
}
```

Theses resource `struct` are packed in `Resources` `struct`s and then placed in the `Context` of
each task.

``` rust
// given: `tasks: { a: { resources: [FOO, BAR] } }`

// app! produces
mod a {
    pub struct Context {
        pub resources: Resources,
        // ..
    }

    pub struct Resources {
        pub FOO: __resource::FOO,
        pub BAR: __resource::BAR,
    }
}
```
