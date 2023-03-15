use std::fmt::Debug;
use std::iter::Peekable;

use crate::{app_subnet_recovery, nns_recovery_failover_nodes, nns_recovery_same_nodes};
use crate::{
    app_subnet_recovery::AppSubnetRecovery, error::RecoveryError,
    nns_recovery_failover_nodes::NNSRecoveryFailoverNodes,
    nns_recovery_same_nodes::NNSRecoverySameNodes, steps::Step, RecoveryResult,
};
use slog::{info, warn, Logger};

pub trait RecoveryIterator<StepType: Copy + Debug + PartialEq, I: Iterator<Item = StepType>> {
    fn get_step_iterator(&mut self) -> &mut Peekable<I>;
    fn get_step_impl(&self, step_type: StepType) -> RecoveryResult<Box<dyn Step>>;
    fn store_next_step(&mut self, step_type: Option<StepType>);

    fn interactive(&self) -> bool;
    fn read_step_params(&mut self, step_type: StepType);
    fn get_logger(&self) -> &Logger;

    /// Advances the iterator to the specified step.
    fn resume(&mut self, step: StepType) {
        while let Some(current_step) = self
            .get_step_iterator()
            .next_if(|current_step| *current_step != step)
        {
            info!(
                self.get_logger(),
                "Skipping already executed step {:?}", current_step
            );
            if current_step == step {
                break;
            }
        }
    }

    fn next_step(&mut self) -> Option<(StepType, Box<dyn Step>)> {
        let result = if let Some(current_step) = self.get_step_iterator().next() {
            if self.interactive() {
                self.read_step_params(current_step);
            }
            match self.get_step_impl(current_step) {
                Ok(step) => Some((current_step, step)),
                Err(RecoveryError::StepSkipped) => {
                    info!(self.get_logger(), "Skipping step {:?}", current_step);
                    self.next_step()
                }
                Err(e) => {
                    warn!(
                        self.get_logger(),
                        "Step generation of {:?} failed: {}", current_step, e
                    );
                    warn!(self.get_logger(), "Skipping step...");
                    self.next_step()
                }
            }
        } else {
            None
        };

        let next_step = self.get_step_iterator().peek().copied();
        self.store_next_step(next_step);
        result
    }
}

impl Iterator for AppSubnetRecovery {
    type Item = (app_subnet_recovery::StepType, Box<dyn Step>);
    fn next(&mut self) -> Option<Self::Item> {
        self.next_step()
    }
}

impl Iterator for NNSRecoverySameNodes {
    type Item = (nns_recovery_same_nodes::StepType, Box<dyn Step>);
    fn next(&mut self) -> Option<Self::Item> {
        self.next_step()
    }
}

impl Iterator for NNSRecoveryFailoverNodes {
    type Item = (nns_recovery_failover_nodes::StepType, Box<dyn Step>);
    fn next(&mut self) -> Option<Self::Item> {
        self.next_step()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeStep {}

    impl Step for FakeStep {
        fn descr(&self) -> String {
            String::from("Fake Step Description")
        }

        fn exec(&self) -> RecoveryResult<()> {
            Ok(())
        }
    }

    /// Fake RecoveryIterator which iterates from 0 to 9.
    struct FakeRecoveryIterator {
        step_iterator: Peekable<std::ops::Range<u64>>,
        logger: Logger,
        read_step_params_called: bool,
        interactive: bool,
        next_step: Option<u64>,
    }

    impl FakeRecoveryIterator {
        fn new(interactive: bool) -> Self {
            Self {
                step_iterator: (0..10).peekable(),
                logger: crate::util::make_logger(),
                read_step_params_called: false,
                interactive,
                next_step: None,
            }
        }
    }

    impl RecoveryIterator<u64, std::ops::Range<u64>> for FakeRecoveryIterator {
        fn get_step_iterator(&mut self) -> &mut Peekable<std::ops::Range<u64>> {
            &mut self.step_iterator
        }

        fn store_next_step(&mut self, next_step: Option<u64>) {
            self.next_step = next_step
        }

        fn get_logger(&self) -> &Logger {
            &self.logger
        }

        fn interactive(&self) -> bool {
            self.interactive
        }

        fn get_step_impl(&self, _step_type: u64) -> RecoveryResult<Box<dyn Step>> {
            Ok(Box::new(FakeStep {}))
        }
        fn read_step_params(&mut self, _step_type: u64) {
            self.read_step_params_called = true;
        }
    }

    #[test]
    fn resume_advances_to_right_step() {
        let mut fake_recovery_iterator = FakeRecoveryIterator::new(/*interactive=*/ true);

        fake_recovery_iterator.resume(5);

        assert_eq!(fake_recovery_iterator.step_iterator.next(), Some(5));
    }

    #[test]
    fn resume_doesnt_read_step_params() {
        let mut fake_recovery_iterator = FakeRecoveryIterator::new(/*interactive=*/ true);

        fake_recovery_iterator.resume(5);

        assert!(!fake_recovery_iterator.read_step_params_called);
    }

    #[test]
    fn next_step_stores_next_step() {
        let mut fake_recovery_iterator = FakeRecoveryIterator::new(/*interactive=*/ true);

        fake_recovery_iterator.next_step();
        assert_eq!(Some(1), fake_recovery_iterator.next_step);

        fake_recovery_iterator.next_step();
        assert_eq!(Some(2), fake_recovery_iterator.next_step);
    }

    #[test]
    fn next_step_reads_params_only_when_interactive() {
        for &interactive in &[false, true] {
            let mut fake_recovery_iterator = FakeRecoveryIterator::new(interactive);

            fake_recovery_iterator.next_step();

            assert_eq!(fake_recovery_iterator.read_step_params_called, interactive);
        }
    }
}
