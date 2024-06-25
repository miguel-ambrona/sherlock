//! Number of captures rule.
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
use crate::Legality::Illegal;

#[derive(Debug)]
pub struct CapturesBoundsRule {
    nb_captures_counter: usize,
    steady_counter: usize,
}

impl Rule for CapturesBoundsRule {
    fn new() -> Self {
        CapturesBoundsRule {
            nb_captures_counter: 0,
            steady_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.nb_captures_counter = analysis.nb_captures.counter();
        self.steady_counter = analysis.steady.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.nb_captures_counter != analysis.nb_captures.counter()
            || self.steady_counter != analysis.steady.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;
        for color in ALL_COLORS {
            // count the number of missing opponents and add all our lower bounds
            let nb_missing_opponents = 16 - analysis.board.color_combined(!color).popcnt() as i32;
            let sum_lower_bounds: i32 = COLOR_ORIGINS[color.to_index()]
                .map(|square| analysis.nb_captures_lower_bound(square))
                .sum();

            for square in *Board::default().color_combined(color) {
                // steady pieces never moved, thus never captured
                if analysis.is_steady(square) {
                    progress |= analysis.update_captures_upper_bound(square, 0);
                }

                // the number of captures of a piece can be upper bounded by the number of
                // missing enemy pieces minus the captures performed by ally pieces
                let lower = analysis.nb_captures_lower_bound(square);
                let new_upper = nb_missing_opponents - (sum_lower_bounds - lower);
                if new_upper < analysis.nb_captures_upper_bound(square) {
                    progress |= analysis.update_captures_upper_bound(square, new_upper);
                }

                // if the bounds ever become incompatible, the position must be illegal
                if new_upper < lower {
                    analysis.result = Some(Illegal);
                }
            }
        }
        progress
    }
}

#[cfg(test)]
mod tests {

    use chess::Square;

    use super::*;
    use crate::{analysis::Analysis, utils::*, RetractableBoard};

    #[test]
    fn test_nb_captures_rule() {
        // White is missing 10 pieces, Black is missing 8
        let board = RetractableBoard::from_fen("rnbqkbnr/8/8/8/8/8/8/1NBQKBN1 w - -")
            .expect("Valid Position");
        let mut analysis = Analysis::new(&board);
        let captures_rule = CapturesBoundsRule::new();

        let bounds = |analysis: &Analysis, square: Square| -> (i32, i32) {
            (
                analysis.nb_captures_lower_bound(square),
                analysis.nb_captures_upper_bound(square),
            )
        };

        captures_rule.apply(&mut analysis);

        // check that two sources are bounded as expected: (0, #missing_oponents)
        assert_eq!(bounds(&analysis, A1), (0, 8));
        assert_eq!(bounds(&analysis, G8), (0, 10));

        // pretend these are now steady
        analysis.update_steady(bitboard_of_squares(&[A1, G8]));
        captures_rule.apply(&mut analysis);

        // their bounds now contain (0, 0)
        assert_eq!(bounds(&analysis, A1), (0, 0));
        assert_eq!(bounds(&analysis, G8), (0, 0));

        // others are still bounded by normally
        assert_eq!(bounds(&analysis, A2), (0, 8));
        assert_eq!(bounds(&analysis, D8), (0, 10));

        // now, let's pretend B1 has captured at least twice and B8 at least thrice
        analysis.update_captures_lower_bound(B1, 2);
        analysis.update_captures_lower_bound(B8, 3);
        captures_rule.apply(&mut analysis);

        // these squares should have experienced a change in the lower-bound only
        assert_eq!(bounds(&analysis, B1), (2, 8));
        assert_eq!(bounds(&analysis, B8), (3, 10));

        // but the upper-bound of others should have decreased accordingly
        assert_eq!(bounds(&analysis, G1), (0, 6));
        assert_eq!(bounds(&analysis, D8), (0, 7));

        // again, assume we get updated bounds
        analysis.update_captures_lower_bound(B1, 7);
        analysis.update_captures_lower_bound(H8, 5);
        captures_rule.apply(&mut analysis);

        assert_eq!(bounds(&analysis, B1), (7, 8));
        assert_eq!(bounds(&analysis, B8), (3, 5));
        assert_eq!(bounds(&analysis, H8), (5, 7));

        assert_eq!(bounds(&analysis, G1), (0, 1));
        assert_eq!(bounds(&analysis, D8), (0, 2));

        assert_eq!(analysis.result, None);

        // finally, push things beyond the limit and get an illegal position
        analysis.update_captures_lower_bound(F8, 3);
        captures_rule.apply(&mut analysis);

        assert_eq!(analysis.result, Some(Illegal));
    }
}
