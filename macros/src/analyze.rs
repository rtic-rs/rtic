use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use either::Either;
use syn::{Ident, Type};
use syntax::check::App;

pub fn app(app: &App) -> Context {
    let mut async = HashSet::new();
    let mut async_after = HashSet::new();
    let mut dispatchers = HashMap::new();
    let mut triggers = HashMap::new();
    let mut tq = TimerQueue::new();
    let mut free_interrupts = app.free_interrupts.iter().cloned().collect::<Vec<_>>();

    async.extend(&app.init.async);
    async_after.extend(&app.init.async_after);

    // compute dispatchers
    for (name, task) in &app.tasks {
        match task.interrupt_or_capacity {
            Either::Left(interrupt) => {
                triggers.insert(interrupt, (*name, task.priority));
            }
            Either::Right(capacity) => {
                let dispatcher = dispatchers.entry(task.priority).or_insert(Dispatcher::new(
                    free_interrupts.pop().expect("not enough free interrupts"),
                ));
                dispatcher.tasks.push(*name);
                dispatcher.capacity += capacity;
            }
        }

        for task in &task.async {
            async.insert(*task);
        }

        for task in &task.async_after {
            async_after.insert(*task);

            // Timer queue
            if let Entry::Vacant(entry) = tq.tasks.entry(*task) {
                tq.capacity += app.tasks[task].interrupt_or_capacity.right().unwrap();
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

    // async
    for (caller_priority, task) in app.tasks
        .values()
        .flat_map(|caller| caller.async.iter().map(move |task| (caller.priority, task)))
    {
        // async callers contend for the consumer end of the task slot queue (#task::SQ) and ...
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

    // async_after
    for (caller_priority, task) in app.tasks.values().flat_map(|caller| {
        caller
            .async_after
            .iter()
            .map(move |task| (caller.priority, task))
    }) {
        // async_after callers contend for the consumer end of the task slot queue (#task::SQ) and
        // ...
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
        async,
        async_after,
        ceilings,
        dispatchers,
        sys_tick,
        triggers,
        timer_queue: tq,
    }
}

pub struct Context {
    // set of `async` tasks
    pub async: HashSet<Ident>,
    // set of `async_after` tasks
    pub async_after: HashSet<Ident>,
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
