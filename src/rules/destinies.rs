//! Destinies rule.
//!
//! We filter out destinies that are not reachable.

use super::{Analysis, Rule, ALL_ORIGINS};

#[derive(Debug)]
pub struct DestiniesRule {
    origins_counter: usize,
    reachable_counter: usize,
}

impl Rule for DestiniesRule {
    fn new() -> Self {
        DestiniesRule {
            origins_counter: 0,
            reachable_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.origins_counter = analysis.origins.counter();
        self.reachable_counter = analysis.reachable.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.origins_counter != analysis.origins.counter()
            || self.reachable_counter != analysis.reachable.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for square in ALL_ORIGINS {
            let reachable_destinies = analysis.destinies(square) & analysis.reachable(square);
            progress |= analysis.update_destinies(square, reachable_destinies)
        }
        progress
    }
}
