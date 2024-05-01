//! Captures bounds rule.
//!
//! A rule to update the known bounds on the number of captures performed by
//! every piece. Steady pieces never moved, so their bounds are set to (0, 0).
//! The number of captures of non-steady pieces can be upper-bounded by the
//! number of missing opponents minus the sum of all lower bounds of ally
//! pieces.
//! If at any point a lower bound exceeds the corresponding upper bound, the
//! position can be declared to be illegal.

use chess::{Board, ALL_COLORS};

use super::{Analysis, Rule, COLOR_ORIGINS};

#[derive(Debug)]
pub struct CapturesBoundsRule {
    captures_bounds_counter: usize,
    steady_counter: usize,
}

impl Rule for CapturesBoundsRule {
    fn new() -> Self {
        CapturesBoundsRule {
            captures_bounds_counter: 0,
            steady_counter: 0,
        }
    }

    fn is_applicable(&self, state: &Analysis) -> bool {
        self.captures_bounds_counter != state.captures_bounds.counter()
            || self.steady_counter != state.steady.counter()
            || self.captures_bounds_counter == 0
            || self.steady_counter == 0
    }

    fn apply(&mut self, state: &mut Analysis) {
        let mut progress = false;
        for color in ALL_COLORS {
            // count the number of missing opponents and add all our lower bounds
            let nb_missing_opponents = 16 - state.board.color_combined(!color).popcnt() as i32;
            let sum_lower_bounds: i32 = COLOR_ORIGINS[color.to_index()]
                .map(|square| state.nb_captures_lower_bound(square))
                .sum();

            for square in *Board::default().color_combined(color) {
                // steady pieces never moved, thus never captured
                if state.is_steady(square) {
                    state.update_captures_upper_bound(square, 0);
                }

                // the number of captures of a piece can be upper bounded by the number of
                // missing enemy pieces minus the captures performed by ally pieces
                let lower = state.nb_captures_lower_bound(square);
                let new_upper = nb_missing_opponents - (sum_lower_bounds - lower);
                if new_upper < state.nb_captures_upper_bound(square) {
                    state.update_captures_upper_bound(square, new_upper);
                    progress = true;
                }

                // if the bounds ever become incompatible, the position must be illegal
                if new_upper < lower {
                    state.illegal = Some(true);
                }
            }
        }

        // update the rule state
        self.captures_bounds_counter = state.captures_bounds.counter();
        self.steady_counter = state.steady.counter();

        // report any progress
        state.captures_bounds.increase_counter(progress);
        state.progress |= progress;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chess::{Board, Square};

    use super::*;
    use crate::{rules::Rule, utils::*};

    #[test]
    fn test_captures_bounds_rule() {
        // White is missing 10 pieces, Black is missing 8
        let board = Board::from_str("rnbqkbnr/8/8/8/8/8/8/1NBQKBN1 w - -").expect("Valid Position");
        let mut state = Analysis::new(&board);
        let mut captures_rule = CapturesBoundsRule::new();

        let bounds = |state: &Analysis, square: Square| -> (i32, i32) {
            (
                state.nb_captures_lower_bound(square),
                state.nb_captures_upper_bound(square),
            )
        };

        captures_rule.apply(&mut state);

        // check that two sources are bounded as expected: (0, #missing_oponents)
        assert_eq!(bounds(&state, A1), (0, 8));
        assert_eq!(bounds(&state, G8), (0, 10));

        // pretend these are now steady
        state.update_steady(bitboard_of_squares(&[A1, G8]));
        captures_rule.apply(&mut state);

        // their bounds now contain (0, 0)
        assert_eq!(bounds(&state, A1), (0, 0));
        assert_eq!(bounds(&state, G8), (0, 0));

        // others are still bounded by normally
        assert_eq!(bounds(&state, A2), (0, 8));
        assert_eq!(bounds(&state, D8), (0, 10));

        // now, let's pretend B1 has captured at least twice and B8 at least thrice
        state.update_captures_lower_bound(B1, 2);
        state.update_captures_lower_bound(B8, 3);
        captures_rule.apply(&mut state);

        // these squares should have experienced a change in the lower-bound only
        assert_eq!(bounds(&state, B1), (2, 8));
        assert_eq!(bounds(&state, B8), (3, 10));

        // but the upper-bound of others should have decreased accordingly
        assert_eq!(bounds(&state, G1), (0, 6));
        assert_eq!(bounds(&state, D8), (0, 7));

        // again, assume we get updated bounds
        state.update_captures_lower_bound(B1, 7);
        state.update_captures_lower_bound(H8, 5);
        captures_rule.apply(&mut state);

        assert_eq!(bounds(&state, B1), (7, 8));
        assert_eq!(bounds(&state, B8), (3, 5));
        assert_eq!(bounds(&state, H8), (5, 7));

        assert_eq!(bounds(&state, G1), (0, 1));
        assert_eq!(bounds(&state, D8), (0, 2));

        assert_eq!(state.illegal, None);

        // finally, push things beyond the limit and get an illegal position
        state.update_captures_lower_bound(F8, 3);
        captures_rule.apply(&mut state);

        assert_eq!(state.illegal, Some(true));
    }
}
