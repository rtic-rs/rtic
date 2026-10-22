use crate::{argument_parsing::Globals, cargo_command::CargoCommand, run::run_and_convert};

use super::FinalRunResult;

pub trait CoalescingRunner {
    /// Run all the commands in this iterator, and coalesce the results into
    /// one error (if any individual commands failed)
    fn run_and_coalesce(self) -> Vec<FinalRunResult>;
}

impl<'g, I> CoalescingRunner for I
where
    I: Iterator<Item = (&'g Globals, CargoCommand, bool)>,
{
    fn run_and_coalesce(self) -> Vec<FinalRunResult> {
        self.map(run_and_convert).collect()
    }
}
