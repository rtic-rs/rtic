# The timer queue

In this section we explore the *timer queue*, the backbone of the `scheduled_in` API.

The `schedule_in` method schedules a task run in the future. `schedule_in` doesn't directly enqueue
tasks into the ready queues, instead it enqueues them in the *timer queue*. The timer queue is a
priority queue that prioritizes tasks with the nearest scheduled start. Associated to the timer
queue there is an interrupt handler that moves tasks that have become ready into the ready queues.
