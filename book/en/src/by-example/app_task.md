# Defining tasks with `#[task]`

Tasks, defined with `#[task]`, are the main mechanism of getting work done in RTIC. Every task can be spawned, now or later, be sent messages (message passing) and be given priorities for preemptive multitasking.

There are two kinds of tasks, software tasks and hardware tasks, and the difference is that hardware tasks are bound to a specific interrupt vector in the MCU while software tasks are not. This means that if a hardware task is bound to the UART's RX interrupt the task will run every time a character is received.

In the coming pages we will explore both tasks and the different options available.
