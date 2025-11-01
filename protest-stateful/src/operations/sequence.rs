//! Operation sequence generation using Protest generators

use crate::operations::{Operation, OperationSequence};
use protest::GeneratorConfig;
use rand::Rng;

/// Generator for operation sequences
pub struct SequenceGenerator<Op, OpGen> {
    op_generator: OpGen,
    min_length: usize,
    max_length: usize,
    _phantom: std::marker::PhantomData<Op>,
}

impl<Op, OpGen> SequenceGenerator<Op, OpGen>
where
    Op: Operation,
    OpGen: protest::Generator<Op>,
{
    /// Create a new sequence generator
    pub fn new(op_generator: OpGen, min_length: usize, max_length: usize) -> Self {
        Self {
            op_generator,
            min_length,
            max_length,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Generate a sequence respecting preconditions
    pub fn generate_valid<R: Rng>(
        &self,
        rng: &mut R,
        config: &GeneratorConfig,
        initial_state: &mut Op::State,
    ) -> OperationSequence<Op> {
        let length = rng.gen_range(self.min_length..=self.max_length);
        let mut sequence = OperationSequence::new();

        for _ in 0..length {
            // Try to generate a valid operation
            let mut attempts = 0;
            while attempts < 100 {
                let op = self.op_generator.generate(rng, config);
                if op.precondition(initial_state) {
                    op.execute(initial_state);
                    sequence.push(op);
                    break;
                }
                attempts += 1;
            }
        }

        sequence
    }
}

impl<Op, OpGen> protest::Generator<OperationSequence<Op>> for SequenceGenerator<Op, OpGen>
where
    Op: Operation + 'static,
    OpGen: protest::Generator<Op> + 'static,
{
    fn generate(
        &self,
        rng: &mut dyn rand::RngCore,
        config: &GeneratorConfig,
    ) -> OperationSequence<Op> {
        let length = if self.min_length == self.max_length {
            self.min_length
        } else {
            rng.gen_range(self.min_length..=self.max_length)
        };
        let mut operations = Vec::new();

        for _ in 0..length {
            operations.push(self.op_generator.generate(rng, config));
        }

        OperationSequence::from_vec(operations)
    }

    fn shrink(
        &self,
        _value: &OperationSequence<Op>,
    ) -> Box<dyn Iterator<Item = OperationSequence<Op>>> {
        // Shrinking is handled by OperationSequence::shrink()
        Box::new(std::iter::empty())
    }
}
