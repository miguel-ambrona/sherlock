//! Unretractable rule.
//!
//! The notion of unretractable piece is similar to the one of steady pieces.
//! However, pieces are not steady unless proven otherwise, whereas we say
//! pieces are unretractable unless proven otherwise.
//! That is, all pieces are assumed to be unretractable by default and they
//! are labeled as "retractable" as soon as they have a predecessor square
//! that is empty or occupied by a retractable piece.
//!
//! When no more pieces can be labeled, we can conclude that the remaining
//! ones really are unretractable. If there exist unretractable pieces that are
//! not in their starting square, the position must be illegal.

use chess::{BitBoard, ALL_COLORS, EMPTY};

use super::{Analysis, Rule};
use crate::{utils::predecessors, Legality, RetractableBoard};

#[derive(Debug)]
pub struct UnretractableRule {
    steady_counter: usize,
}

impl Rule for UnretractableRule {
    fn new() -> Self {
        UnretractableRule { steady_counter: 0 }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.steady_counter = analysis.steady.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.steady_counter != analysis.steady.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let unretractable = unretractable_pieces(&analysis.board, &analysis.steady.value);

        if unretractable & !analysis.steady.value != EMPTY {
            analysis.result = Some(Legality::Illegal);
        }

        false
    }
}

/// Gets a `Board` and a set of pieces known to be steady and returns all the
/// pieces that cannot possibly retract because they do not have a preceeding
/// square that is empty or occupied by a retractable piece.
fn unretractable_pieces(board: &RetractableBoard, steady: &BitBoard) -> BitBoard {
    let mut retractable = !board.combined();

    loop {
        let retractable_at_start = retractable;

        for color in ALL_COLORS {
            for square in *board.color_combined(color) & !retractable & !steady {
                let piece = board.piece_on(square).unwrap();
                let preds = predecessors(piece, color, square);

                if preds & retractable != EMPTY {
                    retractable |= BitBoard::from_square(square);
                }
            }
        }

        if retractable == retractable_at_start {
            break;
        }
    }
    !retractable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;

    #[test]
    fn test_unretractable_pieces() {
        [
            (
                "4k3/8/8/8/8/4P3/1K1PRP2/4b3 b - -",
                vec![E1, D2, E2, F2, E3],
            ),
            ("4k3/8/8/8/8/1P6/bPP5/1b2K3 b - -", vec![B1, A2, B2, C2, B3]),
            (
                "5bbq/4prkb/5prp/6p1/8/8/8/4K3 b - -",
                vec![G5, F6, G6, H6, E7, F7, G7, H7, F8, G8, H8],
            ),
            ("4k2B/6pr/7p/8/8/8/8/4K3 b - -", vec![H6, G7, H7, H8]),
        ]
        .iter()
        .for_each(|(fen, expected)| {
            let board = RetractableBoard::from_fen(fen).expect("Valid Position");
            assert_eq!(
                unretractable_pieces(&board, &EMPTY),
                bitboard_of_squares(expected)
            );
        })
    }
}
