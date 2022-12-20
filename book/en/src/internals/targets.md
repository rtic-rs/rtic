# Target Architecture

While RTIC can currently target all Cortex-m devices there are some key architecure differences that 
users should be aware of. Namely the absence of Base Priority Mask Register (`BASEPRI`) which lends itself exceptionally well to the hardware priority ceiling support used in RTIC, in the
ARMv6-M and ARMv8-M-base architectures, which forces RTIC to use source masking instead. For each implementation of lock and a detailed commentary of pros and cons, see the implementation of [lock in src/export.rs][src_export].

[src_export]: https://github.com/rtic-rs/cortex-m-rtic/blob/master/src/export.rs

These differences influence how critical sections are realized, but functionality should be the same except that ARMv6-M/ARMv8-M-base cannot have tasks with shared resources bound to exception handlers, as these cannot be masked in hardware.

Table 1 below shows a list of Cortex-m processors and which type of critical section they employ.

#### *Table 1: Critical Section Implementation by Processor Architecture*

| Processor  | Architecture | Priority Ceiling | Source Masking |
| :--------- | :----------: | :--------------: | :------------: |
| Cortex-M0  | ARMv6-M      |                  |     &#2713     |
| Cortex-M0+ | ARMv6-M      |                  |     &#2713     |
| Cortex-M3  | ARMv7-M      |      &#2713      |                |
| Cortex-M4  | ARMv7-M      |      &#2713      |                |
| Cortex-M23 | ARMv8-M-base |                  |     &#2713     |
| Cortex-M33 | ARMv8-M-main |      &#2713      |                |
| Cortex-M7  | ARMv7-M      |      &#2713      |                |

## Priority Ceiling

This implementation is covered in depth by Chapter 4.5 of this book.

## Source Masking

Without a `BASEPRI` register which allows for directly setting a priority ceiling in the Nested 
Vectored Interrupt Controller (NVIC), RTIC must instead rely on disabling (masking) interrupts.

Consider Figure 1 below, 
showing two tasks A and B where A has higher priority but shares a resource with B. 

#### *Figure 1: Shared Resources and Source Masking*

```text
  ┌────────────────────────────────────────────────────────────────┐
  │                                                                │
  │                                                                │
3 │                   Pending    Preempts                          │
2 │             ↑- - -A- - - - -↓A─────────►                       │
1 │          B───────────────────► - - - - B────────►              │
0 │Idle┌─────►                             Resumes  ┌────────►     │
  ├────┴────────────────────────────────────────────┴──────────────┤
  │                                                                │
  └────────────────────────────────────────────────────────────────┴──► Time
                t1    t2        t3         t4
```

At time *t1*, task B locks the shared resource by selectively disabling all other tasks which share 
the resource using the NVIC. In effect this raises the virtual priority ceiling. Task A is one such
task that shares resources with task B. At time *t2*, task A is either spawned by task B or becomes 
pending through an interrupt condition, but does not yet preempt task B even though its priority is 
greater. This is because the NVIC is preventing it from starting due to task A's source mask being 
disabled. At time *t3*, task B releases the lock by re-enabling the tasks in the NVIC. Because 
task A was pending and has a higher priority than task B, it immediately preempts task B and is 
free to use the shared resource without risk of data race conditions. At time *t4*, task A completes
and returns the execution context to B.

Since source masking relies on use of the NVIC, core exception sources such as HardFault, SVCall,
PendSV, and SysTick cannot share data with other tasks.
