//! Refine origins rule.
//!
//! This rule exploits the fact that if there is a group of k pieces with k
//! combined candidate origins, those origins cannot be origins of any other
//! piece.

use chess::{Board, ALL_COLORS};

use super::{Rule, State};
use crate::utils::find_k_group;

#[derive(Debug)]
pub struct RefineOriginsRule {
    origins_counter: usize,
}

impl Rule for RefineOriginsRule {
    fn new(_board: &Board) -> Self {
        RefineOriginsRule { origins_counter: 0 }
    }

    fn is_applicable(&self, state: &State) -> bool {
        self.origins_counter != state.origins.counter() || self.origins_counter == 0
    }

    fn apply(&mut self, state: &mut State) {
        let mut progress = false;

        for color in ALL_COLORS {
            // We iterate up to k = 10, since that is the maximum number of candidate
            // origins of any piece after applying the origins rule.
            for k in 1..=10 {
                let mut iter = *state.board.color_combined(color);
                loop {
                    match find_k_group(k, &state.get_origins(), iter) {
                        None => break,
                        Some((group, remaining)) => {
                            iter = remaining;
                            for square in iter {
                                let square_origins = state.origins(square) & !group;
                                progress |= state.origins(square) != square_origins;
                                state.update_origins(square, square_origins);
                            }
                        }
                    }
                }
            }
        }

        // update the rule state and report any progress
        self.origins_counter = state.origins.counter();
        if progress {
            state.origins.increase_counter();
            state.progress = true;
        }
    }
}
