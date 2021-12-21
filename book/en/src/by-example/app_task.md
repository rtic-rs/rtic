# Defining tasks with `#[task]`

Tasks, defined with `#[task]`, are the main mechanism of getting work done in RTIC.

Tasks can

* Be spawned (now or in the future)
* Receive messages (message passing)
* Prioritized allowing preemptive multitasking
* Optionally bind to a hardware interrupt

RTIC makes a distinction between “software tasks” and “hardware tasks”.
Hardware tasks are tasks that are bound to a specific interrupt vector in the MCU while software tasks are not.

This means that if a hardware task is bound to an UART RX interrupt the task will run every
time this interrupt triggers, usually when a character is received.

In the coming pages we will explore both tasks and the different options available.
