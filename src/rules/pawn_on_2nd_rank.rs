//! Pawn on 2nd rank rule.
//!
//! If there is a pawn on their relative 2nd rank, the opponent king cannot
//! possibly have moved to the squares threatened by this pawn. We remove
//! all such moves from the mobility graphs accordingly.

use chess::{get_pawn_attacks, get_rank, Piece, ALL_COLORS, EMPTY};

use super::{Analysis, Rule};

#[derive(Debug)]
pub struct PawnOn2ndRankRule {
    applied: bool,
}

impl Rule for PawnOn2ndRankRule {
    fn new() -> Self {
        PawnOn2ndRankRule { applied: false }
    }

    fn update(&mut self, _analysis: &Analysis) {
        self.applied = true;
    }

    fn is_applicable(&self, _analysis: &Analysis) -> bool {
        !self.applied
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for color in ALL_COLORS {
            for square in analysis.board.color_combined(color)
                & analysis.board.pieces(Piece::Pawn)
                & get_rank(color.to_second_rank())
            {
                for attacked_sq in get_pawn_attacks(square, color, !EMPTY) {
                    progress |= analysis.remove_incoming_edges(Piece::King, !color, attacked_sq);
                }
            }
        }

        progress
    }
}

#[cfg(test)]
mod tests {

    use chess::{Color::*, Piece::*};

    use super::*;
    use crate::{utils::*, RetractableBoard};

    #[test]
    fn test_pawn_on_2nd_rank() {
        let board =
            RetractableBoard::from_fen("rnbqkbnr/1pp1pp2/8/8/8/2PP4/P1P3PP/RNBQKBNR w KQkq -")
                .expect("Valid Position");
        let mut analysis = Analysis::new(&board);

        let pawn_on_2nd_rank = PawnOn2ndRankRule::new();
        pawn_on_2nd_rank.apply(&mut analysis);

        assert!(analysis.mobility.value[White.to_index()][King.to_index()].exists_edge(H5, H6));
        assert!(!analysis.mobility.value[White.to_index()][King.to_index()].exists_edge(H5, G6));
        assert!(!analysis.mobility.value[White.to_index()][King.to_index()].exists_edge(B5, A6));

        assert!(analysis.mobility.value[Black.to_index()][King.to_index()].exists_edge(B4, A3));
        assert!(analysis.mobility.value[Black.to_index()][King.to_index()].exists_edge(F4, E3));
        assert!(!analysis.mobility.value[Black.to_index()][King.to_index()].exists_edge(E3, D3));
    }
}
