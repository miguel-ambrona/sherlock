//! Route to destinies rule.
//!
//! This rule makes sure that for every piece in the starting array, there
//! exists a path from its original square to its candidate destinies.
//! Unreachable destinies are filtered out.

use chess::{BitBoard, Board, ALL_COLORS, EMPTY};

use super::{Rule, COLOR_ORIGINS};
use crate::{analysis::Analysis, utils::distance_to_target};

#[derive(Debug)]
pub struct RouteToDestiniesRule {
    mobility_counter: usize,
}

impl Rule for RouteToDestiniesRule {
    fn new() -> Self {
        Self {
            mobility_counter: 0,
        }
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.mobility_counter != analysis.mobility.counter() || self.mobility_counter == 0
    }

    fn apply(&mut self, analysis: &mut Analysis) {
        let mut progress = false;

        for color in ALL_COLORS {
            for square in COLOR_ORIGINS[color.to_index()] & !analysis.steady.value {
                let piece = Board::default().piece_on(square).unwrap();
                let nb_allowed_captures = analysis.nb_captures_upper_bound(square);
                let mut reachable_destinies = EMPTY;
                for destiny in analysis.destinies(square) {
                    match distance_to_target(
                        &analysis.mobility.value,
                        square,
                        destiny,
                        piece,
                        color,
                    ) {
                        None => (),
                        Some(n) => {
                            if n <= nb_allowed_captures as u32 {
                                reachable_destinies |= BitBoard::from_square(destiny);
                            }
                        }
                    }
                }
                progress |= analysis.update_destinies(square, reachable_destinies);
            }
        }

        // update the rule state
        self.mobility_counter = analysis.mobility.counter();

        // report any progress
        analysis.destinies.increase_counter(progress);
        analysis.progress |= progress;
    }
}
