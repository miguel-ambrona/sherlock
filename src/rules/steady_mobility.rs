//! Steady mobility rule.
//!
//! We refine the mobility graphs based on the information on steady pieces:
//!  - No piece may have passed through a steady-piece square.
//!  - No piece may have moved from a square that was checking a steady king.

use chess::{ALL_COLORS, ALL_PIECES};

use super::{Analysis, Rule};

#[derive(Debug)]
pub struct SteadyMobilityRule {
    steady_counter: usize,
}

impl Rule for SteadyMobilityRule {
    fn new() -> Self {
        SteadyMobilityRule { steady_counter: 0 }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.steady_counter = analysis.steady.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.steady_counter != analysis.steady.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        // Remove all arrows that pass through a steady piece
        for square in analysis.steady.value {
            for color in ALL_COLORS {
                for piece in ALL_PIECES {
                    progress |= analysis.remove_incoming_edges(piece, color, square);
                    progress |= analysis.remove_outgoing_edges(piece, color, square);
                    progress |= analysis.remove_edges_passing_through_square(piece, color, square);
                }
            }
        }

        progress
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chess::{get_rank, Board, Color::*, Piece::*, Rank};

    use super::*;
    use crate::{rules::Rule, utils::*};

    #[test]
    fn test_steady_mobility_rule() {
        let board = Board::from_str("1k6/8/8/8/8/8/8/K7 w - -").expect("Valid Position");
        let mut analysis = Analysis::new(&board);
        let steady_mobility = SteadyMobilityRule::new();

        steady_mobility.apply(&mut analysis);

        // any square should be reachable from H1 for a white rook
        assert_eq!(
            distance_to_target(&analysis.mobility.value, H1, H8, Rook, White),
            Some(0)
        );

        // learn that H7 is steady
        analysis.update_steady(bitboard_of_squares(&[H7]));
        steady_mobility.apply(&mut analysis);

        // H8 should still be reachable, not directly, but reachable
        assert_eq!(
            distance_to_target(&analysis.mobility.value, H1, H8, Rook, White),
            Some(0)
        );

        // not learn that the whole 7th rank is steady
        analysis.update_steady(get_rank(Rank::Seventh));
        steady_mobility.apply(&mut analysis);

        // H8 should no longer be reachable
        assert_eq!(
            distance_to_target(&analysis.mobility.value, H1, H8, Rook, White),
            None
        );
    }
}
