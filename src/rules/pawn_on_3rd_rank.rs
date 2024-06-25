//! Pawn on 3rd rank rule.
//!
//! If there is a pawn on their relative 3rd rank with a single candidate
//! origin, no other piece can possibly have moved between such origin and its
//! current square. We remove all such moves from the mobility graphs
//! accordingly.

use chess::{get_rank, Color, Piece, Rank, ALL_COLORS, ALL_PIECES};

use super::{Analysis, Rule};

#[derive(Debug)]
pub struct PawnOn3rdRankRule {
    origins_counter: usize,
}

impl Rule for PawnOn3rdRankRule {
    fn new() -> Self {
        PawnOn3rdRankRule { origins_counter: 0 }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.origins_counter = analysis.origins.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.origins_counter != analysis.origins.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for color in ALL_COLORS {
            let third_rank = match color {
                Color::White => Rank::Third,
                Color::Black => Rank::Sixth,
            };
            for square in analysis.board.color_combined(color)
                & analysis.board.pieces(Piece::Pawn)
                & get_rank(third_rank)
            {
                // if the 3rd rank pawn has a single candidate origin
                if analysis.origins(square).popcnt() == 1 {
                    let origin = analysis.origins(square).to_square();

                    // remove all arrows that pass through that origin and its current square
                    for other_color in ALL_COLORS {
                        for piece in ALL_PIECES {
                            if piece != Piece::Pawn || other_color != color {
                                progress |= analysis.remove_edges_passing_through_squares(
                                    piece,
                                    other_color,
                                    origin,
                                    square,
                                );
                            }
                        }
                    }
                }
            }
        }

        progress
    }
}

#[cfg(test)]
mod tests {

    use chess::{BitBoard, Color::*, Piece::*};

    use super::*;
    use crate::{utils::*, RetractableBoard};

    #[test]
    fn test_pawn_on_3rd_rank() {
        let board =
            RetractableBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/2P5/P1PPPPPP/RNBQKBNR w KQkq -")
                .expect("Valid Position");
        let mut analysis = Analysis::new(&board);

        let pawn_on_3rd_rank = PawnOn3rdRankRule::new();
        pawn_on_3rd_rank.apply(&mut analysis);

        // the connection between A1 and H8 should be enabled for white bishops
        assert!(analysis.mobility.value[White.to_index()][Bishop.to_index()].exists_edge(A1, H8));

        // learn that B2 is the only origin of the pawn on C3
        analysis.update_origins(C3, BitBoard::from_square(B2));
        pawn_on_3rd_rank.apply(&mut analysis);

        // the connections between B2 and C3 should now be disabled
        assert!(!analysis.mobility.value[White.to_index()][Bishop.to_index()].exists_edge(A1, H8));
        assert!(!analysis.mobility.value[White.to_index()][Bishop.to_index()].exists_edge(B2, C3));
        assert!(!analysis.mobility.value[White.to_index()][Bishop.to_index()].exists_edge(B2, C4));
        assert!(!analysis.mobility.value[White.to_index()][Bishop.to_index()].exists_edge(A1, C3));

        // but for a white pawn the connection B2 -> C3 should still be enabled
        assert!(analysis.mobility.value[White.to_index()][Pawn.to_index()].exists_edge(B2, C3));
    }
}
