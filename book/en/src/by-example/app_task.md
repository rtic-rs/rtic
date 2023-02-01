<!-- Should probably be removed -->

# Defining tasks with `#[task]`

Tasks, defined with `#[task]`, are the main mechanism of getting work done in RTIC.

Tasks can

* Be spawned (now or in the future, also by themselves)
* Receive messages (passing messages between tasks)
* Be prioritized, allowing preemptive multitasking
* Optionally bind to a hardware interrupt

RTIC makes a distinction between “software tasks” and “hardware tasks”.

*Hardware tasks* are tasks that are bound to a specific interrupt vector in the MCU while software tasks are not.

This means that if a hardware task is bound to, lets say, a UART RX interrupt, the task will be run every
time that interrupt triggers, usually when a character is received.

*Software tasks* are explicitly spawned in a task, either immediately or using the Monotonic timer mechanism. 

In the coming pages we will explore both tasks and the different options available.
