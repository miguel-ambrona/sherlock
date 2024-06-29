//! Material rule.

use std::cmp::max;

use chess::{Piece, ALL_COLORS};

use super::Rule;
use crate::{
    analysis::Analysis,
    utils::{DARK_SQUARES, LIGHT_SQUARES},
    Legality::Illegal,
    RetractableBoard,
};

/// A rule that performs a simple check on the position material,
/// making sure it is plausible for an actual game.
/// This rule may be subsumed by other rules, but having it can lead to a
/// quicker identification of illegal positions.
/// This is a one-time rule that will only be applied at the very beginning of
/// the legality analysis.
#[derive(Debug)]
pub struct MaterialRule {
    applied: bool,
}

impl Rule for MaterialRule {
    fn new() -> Self {
        MaterialRule { applied: false }
    }

    fn update(&mut self, _analysis: &Analysis) {
        self.applied = true;
    }

    fn is_applicable(&self, _analysis: &Analysis) -> bool {
        !self.applied
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        if illegal_material(&analysis.board) {
            analysis.result = Some(Illegal);
            true
        } else {
            false
        }
    }
}

/// Returns `true` iff the given board contains an amount of material that is
/// impossible to reach in a legal game.
#[inline]
pub fn illegal_material(board: &RetractableBoard) -> bool {
    for color in ALL_COLORS {
        let pawns = board.pieces(Piece::Pawn) & board.color_combined(color);
        let knights = board.pieces(Piece::Knight) & board.color_combined(color);
        let bishops = board.pieces(Piece::Bishop) & board.color_combined(color);
        let rooks = board.pieces(Piece::Rook) & board.color_combined(color);
        let queens = board.pieces(Piece::Queen) & board.color_combined(color);
        let lower_bound_promoted = max(0, knights.popcnt() as i32 - 2)
            + max(0, (bishops & LIGHT_SQUARES).popcnt() as i32 - 1)
            + max(0, (bishops & DARK_SQUARES).popcnt() as i32 - 1)
            + max(0, rooks.popcnt() as i32 - 2)
            + max(0, queens.popcnt() as i32 - 1);
        if 8 - (pawns.popcnt() as i32) < lower_bound_promoted {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_illegal_material() {
        [
            ("4k3/8/8/8/4N3/8/QQQQQQQQ/3QK3 b - -", false),
            ("4k3/8/8/8/4P3/8/QQQQQQQQ/3QK3 b - -", true),
            ("4k3/8/8/8/3NNN2/8/QQQQQQQQ/3QK3 b - -", true),
            ("rnbqkbnr/ppppppp1/8/2b2b2/8/8/8/K7 w - -", true),
            ("rnbqkbnr/1pppppp1/8/2b2b2/8/8/8/K7 w - -", false),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBRR w - -", true),
            ("rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBRR w - -", false),
            ("4k3/8/8/8/8/2B1B1B1/1B1B1B1B/B1BKB3 b - -", true),
            ("4k3/8/8/8/8/2B1B1B1/1B1B1B1B/B1BK1B2 b - -", false),
        ]
        .iter()
        .for_each(|(fen, expected)| {
            let board = RetractableBoard::from_fen(fen).expect("Valid Position");
            assert_eq!(illegal_material(&board), *expected);
        })
    }
}
