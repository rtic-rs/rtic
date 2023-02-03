<div align="center"><img width="300" height="300" src="RTIC.svg"></div>
<div style="font-size: 6em; font-weight: bolder;" align="center">RTIC</div>

<h1 align="center">The hardware accelerated Rust RTOS</h1>

<p align="center">A concurrency framework for building real-time systems</p>

# Preface

This book contains user level documentation for the Real-Time Interrupt-driven Concurrency
(RTIC) framework. The API reference is available [here](../../api/).

<!-- Formerly known as Real-Time For the Masses. -->

<!--There is a translation of this book in [Russian].-->

<!--[Russian]: ../ru/index.html-->

This is the documentation for RTIC v2.x. 

## RTIC - The Past, current and Future

This section gives a background to the RTIC model. Feel free to skip to section [RTIC the model](preface.md#rtic-the-model) for a TL;DR.

The RTIC framework takes the outset from real-time systems research at Luleå University of Technology (LTU) Sweden. RTIC is inspired by the concurrency model of the [Timber] language, the [RTFM-SRP] based scheduler, the [RTFM-core] language and [Abstract Timer] implementation. For a full list of related research see [TODO].

[Timber]: https://timber-lang.org/
[RTFM-SRP]: https://www.diva-portal.org/smash/get/diva2:1005680/FULLTEXT01.pdf
[RTFM-core]: https://ltu.diva-portal.org/smash/get/diva2:1013248/FULLTEXT01.pdf
[Abstract Timer]: https://ltu.diva-portal.org/smash/get/diva2:1013030/FULLTEXT01.pdf

## Stack Resource Policy based Scheduling

[Stack Resource Policy (SRP)][SRP] based concurrency and resource management is at heart of the RTIC framework. The SRP model itself extends on [Priority Inheritance Protocols], and provides a set of outstanding properties for single core scheduling. To name a few:

- preemptive, deadlock and race-free scheduling
- resource efficiency
  - tasks execute on a single shared stack
  - tasks run-to-completion with wait free access to shared resources
- predictable scheduling, with bounded priority inversion by a single (named) critical section
- theoretical underpinning amenable to static analysis (e.g., for task response times and overall schedulability)

SRP comes with a set of system wide requirements:
- each task is associated a static priority,
- tasks execute on a single-core,  
- tasks must be run-to-completion, and
- resources must be claimed/locked in LIFO order.

[SRP]: https://link.springer.com/article/10.1007/BF00365393
[Priority Inheritance Protocols]: https://ieeexplore.ieee.org/document/57058

## SRP analysis

SRP based scheduling requires the set of static priority tasks and their access to shared resources to be known in order to compute a static *ceiling* (𝝅) for each resource. The static resource *ceiling* 𝝅(r) reflects the maximum static priority of any task that accesses the resource `r`. 

### Example

Assume two tasks `t1` (with priority `p(t1) = 2`) and `t2` (with priority `p(t2) = 4`) both accessing the shared resource `R`. The static ceiling of `R` is 4 (computed from `𝝅(R) = max(p(t1) = 2, p(t2) = 4) = 4`).  

A graph representation of the example:

```mermaid
graph LR
    t1["p(t1) = 2"] --> R
    t2["p(t2) = 4"] --> R
    R["𝝅(R) = 4"]
```

## RTIC the hardware accelerated real-time scheduler

SRP itself is compatible with both dynamic and static priority scheduling. For the implementation of RTIC we leverage on the underlying hardware for accelerated static priority scheduling.

In the case of the `ARM Cortex-M` architecture, each interrupt vector entry `v[i]` is associated a function pointer (`v[i].fn`), and a static priority (`v[i].priority`), an enabled- (`v[i].enabled`) and a pending-bit (`v[i].pending`). 

An interrupt `i` is scheduled (run) by the hardware under the conditions:
1. is `pended` and `enabled` and has a priority higher than the (optional `BASEPRI`) register, and
1. has the highest priority among interrupts meeting 1.

The first condition (1) can be seen a filter allowing RTIC to take control over which tasks should be allowed to start (and which should be prevented from starting).

The SPR model for single-core static scheduling on the other hand states that a task should be scheduled (run) under the conditions:
1. it is `requested` to run and has a static priority higher than the current system ceiling (𝜫)
1. it has the highest static priority among tasks meeting 1.

The similarities are striking and it is not by chance/luck/coincidence. The hardware was cleverly designed with real-time scheduling in mind. 

In order to map the SRP scheduling onto the hardware we need to take a closer look at the system ceiling (𝜫). Under SRP 𝜫 is computed as the maximum priority ceiling of the currently held resources, and will thus change dynamically during the system operation.

## Example

Assume the task model above. Starting from an idle system, 𝜫 is 0, (no task is holding any resource). Assume that `t1` is requested for execution, it will immediately be scheduled. Assume that `t1` claims (locks) the resource `R`. During the claim (lock of `R`) any request `t2` will be blocked from starting (by 𝜫 = `max(𝝅(R) = 4) = 4`, `p(t2) = 4`, thus SRP scheduling condition 1 is not met).

## Mapping

The mapping of static priority SRP based scheduling to the Cortex M hardware is straightforward:

- each task `t` is mapped to an interrupt vector index `i` with a corresponding function `v[i].fn = t` and given the static priority `v[i].priority = p(t)`. 
- the current system ceiling is mapped to the `BASEPRI` register or implemented through masking the interrupt enable bits accordingly.

## Example

For the running example, a snapshot of the ARM Cortex M [Nested Vectored Interrupt Controller (NVIC)][NVIC] may have the following configuration (after task `A` has been pended for execution.)

| Index | Fn  | Priority | Enabled | Pended |
| ----- | --- | -------- | ------- | ------ |
| 0     | t1  | 2        | true    | true   |
| 1     | t2  | 4        | true    | false  |

[NVIC]: https://developer.arm.com/documentation/ddi0337/h/nested-vectored-interrupt-controller/about-the-nvic

(As discussed later, the assignment of interrupt and exception vectors is up to the user.)


A resource claim (lock) will change the current system ceiling (𝜫) and can be implemented as a *named* critical section: 
  - old_ceiling = 𝜫, 𝜫 = 𝝅(r)  
  - execute code within critical section
  - old_ceiling = 𝜫

This amounts to a resource protection mechanism requiring only two machine instructions on enter and one on exit the critical section for managing the `BASEPRI` register. For architectures lacking `BASEPRI`, we can implement the system ceiling through a set of machine instructions for disabling/enabling interrupts on entry/exit for the named critical section. The number of machine instructions vary depending on the number of mask registers that needs to be updated (a single machine operation can operate on up to 32 interrupts, so for the M0/M0+ architecture a single instruction suffice). RTIC will determine the ceiling values and masking constants at compile time, thus all operations are in Rust terms zero-cost.

In this way RTIC fuses SRP based preemptive scheduling with a zero-cost hardware accelerated implementation, resulting in "best in class" guarantees and performance. 

Given that the approach is dead simple, how come SRP and hardware accelerated scheduling is not adopted by any other mainstream RTOS?

The answer is simple, the commonly adopted threading model does not lend itself well to static analysis - there is no known way to extract the task/resource dependencies from the source code at compile time (thus ceilings cannot be efficiently computed and the LIFO resource locking requirement cannot be ensured). Thus SRP based scheduling is in the general case out of reach for any thread based RTOS. 

## RTIC into the Future

Asynchronous programming in various forms are getting increased popularity and language support. Rust natively provides an `async`/`await` API for cooperative multitasking and the compiler generates the necessary boilerplate for storing and retrieving execution contexts (i.e., managing the set of local variables that spans each `await`). 

The Rust standard library provides collections for dynamically allocated data-structures (useful to manage execution contexts at run-time. However, in the setting of resource constrained real-time systems, dynamic allocations are problematic (both regarding performance and reliability - Rust runs into a *panic* on an out-of-memory condition). Thus, static allocation is king!

RTIC provides a mechanism for `async`/`await` that relies solely on static allocations. However, the implementation relies on the `#![feature(type_alias_impl_trait)]` (TAIT) which is undergoing stabilization (thus RTIC 2.0.x currently requires a *nightly* toolchain). Technically, using TAIT, the compiler determines the size of each execution context, which in turn allows these contexts to be statically allocated.

From a modelling perspective `async/await` lifts the run-to-completion requirement of SRP. Each section of code between two yield points (`await`s) can be seen as an individual task. The compiler will reject any attempt to `await` while holding a resource (not doing so would break the strict LIFO requirement on resource usage under SRP).

So with the technical stuff out of the way, what does `async/await` bring to the RTIC table?

The answer is - improved ergonomics! In cases you want a task to perform a sequence of requests and await their results in order to progress. Without `async`/`await` the programmer would be forced to split the task into individual sub-tasks and maintain some sort of state encoding to manually progress. Using `async/await` each yield point (`await`) essentially represents a state, and the progression mechanism is built automatically for you at compile time by means of `Futures`. 

Rust `async`/`await` support is still incomplete and/or under active development (e.g., there are no stable way to express `async` closures, precluding use in iterator patterns). Nevertheless, Rust `async`/`await` is production ready and covers many common use cases. 

An important property is that futures are composable, thus you can await either, all, or any combination of possible futures. This allows e.g., timeouts and/or asynchronous errors to be promptly handled). For more details and examples see Section [todo].

## RTIC the model

An RTIC `app` is a declarative and executable system model for single-core applications, defining a set of (`local` and `shared`) resources operated on by a set of  (`init`, `idle`, *hardware* and *software*) tasks. In short the `init` task runs before any other task and returning a set of initialized resources (`local` and `shared`). Tasks run preemptively based on their associated static priority, `idle` has the lowest priority (and can be used for background work, and/or to put the system to sleep until woken by some event). Hardware tasks are bound to underlying hardware interrupts, while software tasks are scheduled by asynchronous executors (one for each software task priority). 

At compile time the task/resource model is analyzed under SRP and executable code generated with the following outstanding properties:

- guaranteed race-free resource access and deadlock-free execution on a single-shared stack (thanks to SRP)
  - hardware task scheduling is performed directly by the hardware, and
  - software task scheduling is performed by auto generated async executors tailored to the application.

The RTIC API design ensures that both SRP requirements and Rust soundness rules are upheld at all times, thus the executable model is correct by construction. Overall, the generated code infers no additional overhead in comparison to a well crafted hand-written implementation, thus in Rust terms RTIC offers a zero-cost abstraction to concurrency.

## RTIC Interrupt Driven Concurrency

We showcase the RTIC model on a set of increasingly complex examples.

### Example, Simple preemption

Assume a system with two *hardware* tasks, `t1` and `t2` with priorities `p(t1) = 2` and `p(t2) = 4`. `t1` and `t2` are bound to interrupts `INT1` and `INT2` accordingly. 

A trace of the system might look like this:

The system is initially *idle* (no tasks running, thus the `Running Priority` is 0 in figure). At time 1 `INT1` is pended (arrives) and `t1` is dispatched for execution by the hardware, (A) in figure. During the execution of `t1`, `INT2` is pended at time 3. Since `INT2` has higher priority than the currently running interrupt handler (`INT1`) (`Running Priority` is 2 in figure) `t2` is dispatched for execution (preempting the currently running task), (B in figure). At time 5, `t2` is finished and returns, allowing `t1` to resume execution (C in figure).

The scheduling mechanisms are performed by the hardware without any overhead induced by the RTIC framework.

``` wavedrom
{ 
  head:{
   text:'Two Hardware Tasks',
   tick:0,
   every:1
  },  
  signal: [
    [ 'Pend', 
      {
        name: 'INT2', 
        wave: '0..H.l..', 
        node: '...B....' 
      }, 
      {
        name: 'INT1', 
        wave: '0H.....l', 
        node: '.A......' 
      },
    ],
    {
        name: 'Running Priority', 
        wave: '66.6.6.6', 
        
        data: [0, 2, 4, 2, 0]
    }, 
    [ 'Vec',
      {
        name: 'INT2 (p = 4)', 
        wave: '0..3.0..', 
        node: '...D.E..', 
        data: ['t2']
      },
      { 
        name: 'INT1 (p = 2)', 
        wave: '03.2.3.0', 
        node: '.C...F..', 
        data: ['t1','---', 't1']
      },
    ],
  ], 
  edge: [
    'A~>C A',
    'B->D B',
    'E->F C',
  ]
}
```

### Example, Resource locking

Now we assume that tasks `t1` and `t2` share a resource `R`. According to SRP we compute the ceiling value  `𝝅(R) = max(p(t1) = 2, p(t2) = 4) = 4`).

A trace of the system might look like this:

Initially the system is idle and both the `System Ceiling` and the current `Running Priority` is set to 0. At time 1, task `t1` is dispatched as it has highest priority among pended tasks and has a priority higher than both the `System Ceiling` and the current `Running Priority`. At time 2 `t1` locks the resource `R` and the `System Ceiling` is raised to 4 (the maximum ceiling value of all locked resources). At time 3, `INT2` is pended. Task `t2` now has the highest priority among pended tasks and has higher priority then the current  `Running Priority` (which is now 2). However, `t2` has NOT higher priority than the `System Ceiling` (which is now 4), thus `t2` is blocked from dispatch (the grey area in the figure). `t1` continues to execute and releases the lock on `R` at time 4. The `System Ceiling` becomes 0 (no locked resources at this point), which allows the pending task `t2` to be dispatched.

This way SRP guarantees that no two running tasks have concurrent access to the same shared resource. SRP also guarantees that once a task has started to execute, all locks will be granted without awaiting other tasks to progress.

For this specific trace `t1` is blocked 1 time unit. The response time for `t2` is 3 time units (the time since arrival at time 3, until it finishes at time 6).

``` wavedrom
{ 
  head:{
   text:'Two Hardware Tasks with a Shared Resource',
   tick:0,
   every:1
  },  
  signal: [
    [ 'Pend',
      {
        name: 'INT2', 
        wave: '0..H..l.', 
        node: '...A...' 
      }, 
      {
        name: 'INT1', 
        wave: '0H.....l', 
        // node: '.A......' 
      },
    ],
    
      {
        name: 'System Ceiling', 
        wave: '5.5.555.', 
        // node: '..B...',
        data: [0, 4, 0, 4, 0]
      }, 
     
      {
        name: 'Running Priority', 
        wave: '66..6.66', 
        // node: '..B...',
        data: [0, 2, 4, 2, 0]
      }, 

   
    
    [ 'Vec',
      {
        name: 'INT2 (p = 4)', 
        wave: '0..x340.', 
        node: '....B...', 
        data: ['t2', 'L(R)']
      },
      { 
        name: 'INT1 (p = 2)', 
        wave: '034.2.30', 
        node: '.C...F.', 
        data: ['t1','L(R)','---', 't1']
      },
    ],
  ], 
  edge: [
    // 'A~>B A',
  ]
}
```

RTIC implements the SRP system ceiling by means of the BASEPRI register or by interrupt masking. In both cases the scheduling is performed directly by the hardware (the only overhead induced is due constant-time operations to update the BASEPRI/interrupt mask(s) on task dispatch and resource lock/unlock).

Furthermore, [SRP] comes with worst case guarantees allowing to (at compile time) compute the worst case response times for all possible traces of the system. Thus, unlike any mainstream RTOS, RTIC provides the means to implement systems with *hard* real-time requirements.

<!--

[ 'Vec',
      {
        name: 'INT2 (p = 2)', 
        wave: '0..3.0.', 
        node: '...C.E.', 
        data: ["task2"]
      },
      { 
        name: 'INT1 (p = 1)', 
        wave: '03.2.30', 
        node: '.D...F.', 
        data: ['task1','---', 'task2']
      },
    ],

For the documentation older versions, see;

* v1.0.x go [here](/1.0).
* v0.5.x go [here](/0.5).
* v0.4.x go [here](/0.4). -->

<!-- 
{{#include ../../../README.md:7:47}}

{{#include ../../../README.md:48:}} 
-->
