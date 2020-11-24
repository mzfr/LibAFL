use alloc::rc::Rc;
use core::cell::RefCell;
use core::marker::PhantomData;

use crate::corpus::testcase::Testcase;
use crate::engines::State;
use crate::events::{EventManager, NewTestcaseEvent};
use crate::executors::Executor;
use crate::inputs::Input;
use crate::mutators::Mutator;
use crate::stages::Corpus;
use crate::stages::Stage;
use crate::utils::Rand;
use crate::{fire_event, AflError};

// TODO multi mutators stage

pub trait MutationalStage<M, S, EM, E, C, I, R>: Stage<S, EM, E, C, I, R>
where
    M: Mutator<C, I, R>,
    S: State<C, E, I, R>,
    EM: EventManager<S, C, E, I, R>,
    E: Executor<I>,
    C: Corpus<I, R>,
    I: Input,
    R: Rand,
{
    /// The mutator registered for this stage
    fn mutator(&self) -> &M;

    /// The mutator registered for this stage (mutable)
    fn mutator_mut(&mut self) -> &mut M;

    /// Gets the number of iterations this mutator should run for.
    /// This call uses internal mutability, so it may change for each call
    fn iterations(&mut self, rand: &mut R) -> usize {
        1 + rand.below(128) as usize
    }

    /// Runs this (mutational) stage for the given testcase
    fn perform_mutational(
        &mut self,
        rand: &mut R,
        state: &mut S,
        events: &mut EM,
        testcase: Rc<RefCell<Testcase<I>>>,
    ) -> Result<(), AflError> {
        let num = self.iterations(rand);
        for i in 0..num {
            let mut input = testcase.borrow_mut().load_input()?.clone();
            self.mutator_mut()
                .mutate(rand, state.corpus_mut(), &mut input, i as i32)?;

            let (interesting, new_testcase) = state.evaluate_input(input)?;

            self.mutator_mut()
                .post_exec(interesting, new_testcase.clone(), i as i32)?;

            if !new_testcase.is_none() {
                fire_event!(events, NewTestcaseEvent<I>, new_testcase.unwrap())?;
            }
        }
        Ok(())
    }
}

/// The default mutational stage
pub struct StdMutationalStage<M, S, EM, E, C, I, R>
where
    M: Mutator<C, I, R>,
    S: State<C, E, I, R>,
    EM: EventManager<S, C, E, I, R>,
    E: Executor<I>,
    C: Corpus<I, R>,
    I: Input,
    R: Rand,
{
    mutator: M,
    phantom: PhantomData<(S, EM, E, C, I, R)>,
}

impl<M, S, EM, E, C, I, R> MutationalStage<M, S, EM, E, C, I, R>
    for StdMutationalStage<M, S, EM, E, C, I, R>
where
    M: Mutator<C, I, R>,
    S: State<C, E, I, R>,
    EM: EventManager<S, C, E, I, R>,
    E: Executor<I>,
    C: Corpus<I, R>,
    I: Input,
    R: Rand,
{
    /// The mutator, added to this stage
    fn mutator(&self) -> &M {
        &self.mutator
    }

    /// The list of mutators, added to this stage (as mutable ref)
    fn mutator_mut(&mut self) -> &mut M {
        &mut self.mutator
    }
}

impl<M, S, EM, E, C, I, R> Stage<S, EM, E, C, I, R> for StdMutationalStage<M, S, EM, E, C, I, R>
where
    M: Mutator<C, I, R>,
    S: State<C, E, I, R>,
    EM: EventManager<S, C, E, I, R>,
    E: Executor<I>,
    C: Corpus<I, R>,
    I: Input,
    R: Rand,
{
    fn perform(
        &mut self,
        rand: &mut R,
        state: &mut S,
        events: &mut EM,
        testcase: Rc<RefCell<Testcase<I>>>,
    ) -> Result<(), AflError> {
        self.perform_mutational(rand, state, events, testcase)
    }
}

impl<M, S, EM, E, C, I, R> StdMutationalStage<M, S, EM, E, C, I, R>
where
    M: Mutator<C, I, R>,
    S: State<C, E, I, R>,
    EM: EventManager<S, C, E, I, R>,
    E: Executor<I>,
    C: Corpus<I, R>,
    I: Input,
    R: Rand,
{
    /// Creates a new default mutational stage
    pub fn new(mutator: M) -> Self {
        StdMutationalStage {
            mutator: mutator,
            phantom: PhantomData,
        }
    }
}
