use super::FinalRunResult;

pub use iter::*;

pub trait CoalescingRunner<'c> {
    /// Run all the commands in this iterator, and coalesce the results into
    /// one error (if any individual commands failed)
    fn run_and_coalesce(self) -> Vec<FinalRunResult<'c>>;
}

#[cfg(not(feature = "rayon"))]
mod iter {
    use super::*;
    use crate::{argument_parsing::Globals, cargo_command::*, run::run_and_convert};

    pub fn into_iter<T: IntoIterator>(var: T) -> impl Iterator<Item = T::Item> {
        var.into_iter()
    }

    impl<'g, 'c, I> CoalescingRunner<'c> for I
    where
        I: Iterator<Item = (&'g Globals, CargoCommand<'c>, bool)>,
    {
        fn run_and_coalesce(self) -> Vec<FinalRunResult<'c>> {
            self.map(run_and_convert).collect()
        }
    }
}

#[cfg(feature = "rayon")]
mod iter {
    use super::*;
    use crate::{argument_parsing::Globals, cargo_command::*, run::run_and_convert};
    use rayon::prelude::*;

    pub fn into_iter<T: IntoParallelIterator>(var: T) -> impl ParallelIterator<Item = T::Item> {
        var.into_par_iter()
    }

    impl<'g, 'c, I> CoalescingRunner<'c> for I
    where
        I: ParallelIterator<Item = (&'g Globals, CargoCommand<'c>, bool)>,
    {
        fn run_and_coalesce(self) -> Vec<FinalRunResult<'c>> {
            self.map(run_and_convert).collect()
        }
    }
}
