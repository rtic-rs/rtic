#![no_std]

pub trait Post<M> {
    fn post(&mut self, message: M) -> Result<(), M>;
}

pub trait Receive<M> {
    fn receive(&mut self, message: M);
}
