//! Material rule.

use std::cmp::max;

use chess::{Board, ALL_COLORS};

use super::{Rule, State};
use crate::utils::{bishops, knights, pawns, queens, rooks, DARK_SQUARES, LIGHT_SQUARES};

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
    fn new(_board: &Board) -> Self {
        MaterialRule { applied: false }
    }

    fn is_applicable(&self, _state: &State) -> bool {
        !self.applied
    }

    fn apply(&mut self, state: &mut State) {
        self.applied = true;
        if illegal_material(&state.board) {
            state.illegal = Some(true)
        }
    }
}

/// Returns `true` iff the given board contains an amount of material that is
/// impossible to reach in a legal game.
#[inline]
pub fn illegal_material(board: &Board) -> bool {
    for color in ALL_COLORS {
        let bishops = bishops(board, color);
        let lower_bound_promoted = max(0, knights(board, color).popcnt() as i32 - 2)
            + max(0, (bishops & LIGHT_SQUARES).popcnt() as i32 - 1)
            + max(0, (bishops & DARK_SQUARES).popcnt() as i32 - 1)
            + max(0, rooks(board, color).popcnt() as i32 - 2)
            + max(0, queens(board, color).popcnt() as i32 - 1);
        if 8 - (pawns(board, color).popcnt() as i32) < lower_bound_promoted {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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
            let board = Board::from_str(fen).expect("Valid Position");
            assert_eq!(illegal_material(&board), *expected);
        })
    }
}
