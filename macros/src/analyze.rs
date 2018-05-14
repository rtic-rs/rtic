use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use either::Either;
use syn::{Ident, Type};
use syntax::check::App;

pub fn app(app: &App) -> Context {
    let mut schedule_now = HashSet::new();
    let mut schedule_after = HashSet::new();
    let mut dispatchers = HashMap::new();
    let mut triggers = HashMap::new();
    let mut tq = TimerQueue::new();
    let mut free_interrupts = app.free_interrupts.iter().cloned().collect::<Vec<_>>();

    schedule_now.extend(&app.init.schedule_now);

    for task in &app.init.schedule_after {
        schedule_after.insert(*task);

        // Timer queue
        if let Entry::Vacant(entry) = tq.tasks.entry(*task) {
            tq.capacity += app.tasks[task].interrupt_or_instances.right().unwrap();
            entry.insert(app.tasks[task].priority);
        }
    }

    // compute dispatchers
    for (name, task) in &app.tasks {
        match task.interrupt_or_instances {
            Either::Left(interrupt) => {
                triggers.insert(interrupt, (*name, task.priority));
            }
            Either::Right(instances) => {
                let dispatcher = dispatchers.entry(task.priority).or_insert_with(|| {
                    Dispatcher::new(free_interrupts.pop().expect("not enough free interrupts"))
                });
                dispatcher.tasks.push(*name);
                dispatcher.capacity += instances;
            }
        }

        for task in &task.schedule_now {
            schedule_now.insert(*task);
        }

        for task in &task.schedule_after {
            schedule_after.insert(*task);

            // Timer queue
            if let Entry::Vacant(entry) = tq.tasks.entry(*task) {
                tq.capacity += app.tasks[task].interrupt_or_instances.right().unwrap();
                entry.insert(app.tasks[task].priority);
            }
        }
    }

    // The SysTick exception runs at the highest dispatcher priority
    let sys_tick = dispatchers.keys().cloned().max().unwrap_or(1);

    // compute ceilings
    let mut ceilings = Ceilings::new(sys_tick);

    // the SysTick interrupt contends for the timer queue (this has been accounted for in the
    // `Ceilings` constructor) and for the producer end of all the dispatcher queues (__#N::Q)
    for dispatch_priority in dispatchers.keys() {
        ceilings
            .dispatch_queues
            .insert(*dispatch_priority, sys_tick);
    }

    // resources
    for (priority, resource) in app.idle.resources.iter().map(|res| (0, res)).chain(
        app.tasks
            .iter()
            .flat_map(|(name, task)| task.resources.iter().map(move |res| (task.priority, res))),
    ) {
        let ceiling = ceilings
            .resources
            .entry(*resource)
            .or_insert(Ceiling::Owned(priority));
        if priority > (*ceiling).into() {
            *ceiling = Ceiling::Shared(priority);
        } else if priority < (*ceiling).into() && ceiling.is_owned() {
            *ceiling = Ceiling::Shared((*ceiling).into());
        }
    }

    // schedule_now
    for (caller_priority, task) in app.tasks.values().flat_map(|caller| {
        caller
            .schedule_now
            .iter()
            .map(move |task| (caller.priority, task))
    }) {
        // schedule_now callers contend for the consumer end of the task slot queue (#task::SQ) and
        // ..
        let ceiling = ceilings.slot_queues.entry(*task).or_insert(caller_priority);

        if caller_priority > *ceiling {
            *ceiling = caller_priority;
        }

        // .. for the producer end of the dispatcher queue (__#dispatch_priority::Q)
        let dispatch_priority = app.tasks[task].priority;
        let ceiling = ceilings
            .dispatch_queues
            .entry(dispatch_priority)
            .or_insert(dispatch_priority);

        if caller_priority > *ceiling {
            *ceiling = caller_priority;
        }
    }

    // schedule_after
    for (caller_priority, task) in app.tasks.values().flat_map(|caller| {
        caller
            .schedule_after
            .iter()
            .map(move |task| (caller.priority, task))
    }) {
        // schedule_after callers contend for the consumer end of the task slot queue (#task::SQ)
        // and ..
        let ceiling = ceilings.slot_queues.entry(*task).or_insert(caller_priority);

        if caller_priority > *ceiling {
            *ceiling = caller_priority;
        }

        // .. for the timer queue
        if caller_priority > ceilings.timer_queue {
            ceilings.timer_queue = caller_priority;
        }
    }

    Context {
        schedule_now,
        schedule_after,
        ceilings,
        dispatchers,
        sys_tick,
        triggers,
        timer_queue: tq,
    }
}

pub struct Context {
    // set of `schedule_now` tasks
    pub schedule_now: HashSet<Ident>,
    // set of `schedule_after` tasks
    pub schedule_after: HashSet<Ident>,
    pub ceilings: Ceilings,
    // Priority:u8 -> Dispatcher
    pub dispatchers: HashMap<u8, Dispatcher>,
    // Interrupt:Ident -> Task:Ident
    pub triggers: HashMap<Ident, (Ident, u8)>,
    pub timer_queue: TimerQueue,
    // priority of the SysTick exception
    pub sys_tick: u8,
}

pub struct TimerQueue {
    // Task:Ident -> Priority:u8
    tasks: HashMap<Ident, u8>,
    capacity: u8,
}

impl TimerQueue {
    fn new() -> Self {
        TimerQueue {
            tasks: HashMap::new(),
            capacity: 0,
        }
    }

    pub fn capacity(&self) -> u8 {
        self.capacity
    }

    pub fn tasks(&self) -> &HashMap<Ident, u8> {
        &self.tasks
    }
}

pub struct Dispatcher {
    capacity: u8,
    interrupt: Ident,
    tasks: Vec<Ident>,
}

impl Dispatcher {
    fn new(interrupt: Ident) -> Self {
        Dispatcher {
            capacity: 0,
            interrupt,
            tasks: vec![],
        }
    }

    pub fn capacity(&self) -> u8 {
        self.capacity
    }

    pub fn interrupt(&self) -> Ident {
        self.interrupt
    }

    pub fn tasks(&self) -> &[Ident] {
        &self.tasks
    }
}

#[derive(Debug)]
pub struct Ceilings {
    dispatch_queues: HashMap<u8, u8>,
    resources: HashMap<Ident, Ceiling>,
    slot_queues: HashMap<Ident, u8>,
    timer_queue: u8,
}

impl Ceilings {
    fn new(sys_tick_priority: u8) -> Self {
        Ceilings {
            dispatch_queues: HashMap::new(),
            resources: HashMap::new(),
            slot_queues: HashMap::new(),
            timer_queue: sys_tick_priority,
        }
    }

    pub fn dispatch_queues(&self) -> &HashMap<u8, u8> {
        &self.dispatch_queues
    }

    pub fn resources(&self) -> &HashMap<Ident, Ceiling> {
        &self.resources
    }

    pub fn slot_queues(&self) -> &HashMap<Ident, u8> {
        &self.slot_queues
    }

    pub fn timer_queue(&self) -> u8 {
        self.timer_queue
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Ceiling {
    Owned(u8),
    Shared(u8),
}

impl Ceiling {
    pub fn is_owned(&self) -> bool {
        if let Ceiling::Owned(..) = *self {
            true
        } else {
            false
        }
    }
}

impl From<Ceiling> for u8 {
    fn from(ceiling: Ceiling) -> u8 {
        match ceiling {
            Ceiling::Owned(prio) => prio,
            Ceiling::Shared(ceil) => ceil,
        }
    }
}
