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

    fn is_applicable(&self, state: &Analysis) -> bool {
        self.mobility_counter != state.mobility.counter() || self.mobility_counter == 0
    }

    fn apply(&mut self, state: &mut Analysis) {
        let mut progress = false;

        for color in ALL_COLORS {
            for square in COLOR_ORIGINS[color.to_index()] & !state.steady.value {
                let piece = Board::default().piece_on(square).unwrap();
                let nb_allowed_captures = state.nb_captures_upper_bound(square);
                let mut reachable_destinies = EMPTY;
                for destiny in state.destinies(square) {
                    match distance_to_target(&state.mobility.value, square, destiny, piece, color) {
                        None => (),
                        Some(n) => {
                            if n <= nb_allowed_captures as u32 {
                                reachable_destinies |= BitBoard::from_square(destiny);
                            }
                        }
                    }
                }
                progress |= state.update_destinies(square, reachable_destinies);
            }
        }

        // update the rule state
        self.mobility_counter = state.mobility.counter();

        // report any progress
        state.destinies.increase_counter(progress);
        state.progress |= progress;
    }
}
