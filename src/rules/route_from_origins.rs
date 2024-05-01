//! Route from origins rule.
//!
//! This rule makes sure that for every piece on the board there exists a path
//! from its candidate origins to its current square. The candidate origins that
//! do not satisfy this condition are filtered out.

use chess::{BitBoard, Board, EMPTY};

use super::Rule;
use crate::{state::State, utils::distance_from_source};

#[derive(Debug)]
pub struct RouteFromOriginsRule {
    mobility_counter: usize,
}

impl Rule for RouteFromOriginsRule {
    fn new(_board: &Board) -> Self {
        Self {
            mobility_counter: 0,
        }
    }

    fn is_applicable(&self, state: &State) -> bool {
        self.mobility_counter != state.mobility.counter() || self.mobility_counter == 0
    }

    fn apply(&mut self, state: &mut State) {
        let mut progress = false;

        for square in state.board.combined() & !state.steady.value {
            let piece = state.piece_type_on(square);
            let color = state.piece_color_on(square);
            let mut plausible_origins = EMPTY;
            for origin in state.origins(square) {
                let nb_allowed_captures = state.nb_captures_upper_bound(origin);
                match distance_from_source(state, origin, square, piece, color) {
                    None => (),
                    Some(n) => {
                        if n <= nb_allowed_captures as u32 {
                            plausible_origins |= BitBoard::from_square(origin);
                        }
                    }
                }
            }
            progress |= state.update_origins(square, plausible_origins);
        }

        // update the rule state
        self.mobility_counter = state.mobility.counter();

        // report any progress
        state.origins.increase_counter(progress);
        state.progress |= progress;
    }
}
