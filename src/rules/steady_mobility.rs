//! Steady mobility rule.
//!
//! We refine the mobility graphs based on the information on steady pieces:
//!  - No piece may have passed through a steady-piece square.
//!  - No piece may have moved from a square that was checking a steady king.

use chess::{ALL_COLORS, ALL_PIECES};

use super::{Analysis, Rule};
use crate::utils::checking_predecessors;

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

        // Remove all arrows from/into or that pass through a steady piece
        for square in analysis.steady.value {
            for color in ALL_COLORS {
                for piece in ALL_PIECES {
                    progress |= analysis.remove_incoming_edges(piece, color, square);
                    progress |= analysis.remove_outgoing_edges(piece, color, square);
                    progress |= analysis.remove_edges_passing_through_square(piece, color, square);
                }
            }
        }

        // Remove all arrows from a square that checks a steady king
        for king_color in ALL_COLORS {
            let king_square = analysis.board.king_square(king_color);
            if analysis.is_steady(king_square) {
                for piece in ALL_PIECES {
                    for checking_square in checking_predecessors(piece, !king_color, king_square) {
                        progress |=
                            analysis.remove_outgoing_edges(piece, !king_color, checking_square);
                    }
                }
            }
        }

        progress
    }
}

#[cfg(test)]
mod tests {

    use chess::{get_rank, Board, Color::*, Piece::*, Rank};

    use super::*;
    use crate::{rules::Rule, utils::*};

    #[test]
    fn test_steady_pieces() {
        let mut analysis = Analysis::new(&Board::default());
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

    #[test]
    fn test_steady_king() {
        let mut analysis = Analysis::new(&Board::default());
        let steady_mobility = SteadyMobilityRule::new();

        // learn that the black king is steady
        analysis.update_steady(bitboard_of_squares(&[E8]));
        steady_mobility.apply(&mut analysis);

        // make sure a white pawn can still go from E7 to F8
        assert_eq!(
            distance_to_target(&analysis.mobility.value, E7, F8, Pawn, White),
            Some(1)
        );

        // but no white pawn can go from D7 to C8
        assert_eq!(
            distance_to_target(&analysis.mobility.value, D7, C8, Pawn, White),
            None
        );

        // and a white knight can move to F6, but not from F6
        assert_eq!(
            distance_to_target(&analysis.mobility.value, G1, F6, Knight, White),
            Some(0)
        );
        assert_eq!(
            distance_to_target(&analysis.mobility.value, F6, G1, Knight, White),
            None
        );
    }
}
