//! Steady rule.
//!
//! A rule that updates the information on steady pieces: pieces that have
//! certainly never moved and are still on their starting square.
//! Steady pieces can be identified through the castling information or by
//! realizing that a piece is limited in movement by other steady pieces (e.g.
//! pawns on their relative 2nd rank are steady, thus a white bishop on c1 is
//! steady if there are white pawns on b2 and d2).

use chess::{BitBoard, Board, CastleRights, Piece, ALL_COLORS};

use super::{Rule, State};
use crate::utils::predecessors;

#[derive(Debug)]
pub struct SteadyRule {
    steady_counter: usize,
}

impl Rule for SteadyRule {
    fn new(_board: &Board) -> Self {
        SteadyRule { steady_counter: 0 }
    }

    fn is_applicable(&self, state: &State) -> bool {
        self.steady_counter != state.steady.counter() || self.steady_counter == 0
    }

    fn apply(&mut self, state: &mut State) {
        let new_steady = steady_pieces(&state.board, &state.get_steady());
        state.update_steady(new_steady);

        // update the rule state and report any progress
        self.steady_counter = state.steady.counter();
        if new_steady != state.get_steady() {
            state.steady.increase_counter();
            state.progress = true;
        }
    }
}

/// Gets a `Board`` and a `BitBoard` containing the information on squares
/// assumed to contain steady pieces, it returns an updated `BitBoard` of steady
/// pieces.
fn steady_pieces(board: &Board, steady: &BitBoard) -> BitBoard {
    debug_assert!(board.is_sane());
    let mut steady = *steady;
    for color in ALL_COLORS {
        // steady pieces due to castling rights
        let castle_rights = board.castle_rights(color);
        if castle_rights != CastleRights::NoRights {
            steady |= castle_rights.unmoved_rooks(color)
                | (board.pieces(Piece::King) & board.color_combined(color))
        };

        // steady pieces because they are restricted by other steady pieces
        loop {
            let steady_at_start = steady;
            for square in *board.color_combined(color) & !steady {
                let piece = board.piece_on(square).unwrap();
                let preds = predecessors(piece, color, square);

                if (preds & steady) == preds {
                    // all predecessors are steady
                    steady |= BitBoard::from_square(square);
                }
            }
            if steady == steady_at_start {
                break;
            }
        }

        // a king-queen couple surrounded by steady pieces must be steady
        let couple = MARRIAGE_COUPLE[color.to_index()];
        let cage = MARRIAGE_CAGE[color.to_index()];
        if (cage & steady) == cage && (couple & board.color_combined(color)) == couple {
            steady |= couple;
        }
    }
    steady
}

const MARRIAGE_COUPLE: [BitBoard; 2] = [
    BitBoard(24),                  // D1, E1
    BitBoard(1729382256910270464), // D8, E8
];
const MARRIAGE_CAGE: [BitBoard; 2] = [
    BitBoard(15396),               // C1, C2, D2, E2, F2, F1
    BitBoard(2610961883968045056), // C8, C7, D7, E7, F7, F8
];

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_steady_pieces() {
        [
            (
                "r2qkb2/8/8/6p1/6P1/8/1P1P4/2B1K2R w q -",
                vec![],
                vec![C1, B2, D2, A8, E8],
            ),
            (
                "2bqkb2/1pppppp1/p6p/8/4P3/2P5/8/R3K2R w Q -",
                vec![],
                vec![A1, E1, B7, C7, D7, E7, F7, G7, C8, D8, E8, F8],
            ),
            (
                "2bqkb2/1ppppp2/8/8/8/8/4P1P1/R3K2R w - -",
                vec![],
                vec![E2, G2, B7, C7, D7, E7, F7, C8],
            ),
            (
                "1n2k3/8/8/8/8/8/6P1/4K2B w - -",
                vec![A6, C6, D7],
                vec![G2, H1, B8],
            ),
            (
                "k7/8/8/8/8/8/4P1PP/K5NR w - -",
                vec![F3, H3],
                vec![G1, H1, E2, G2, H2],
            ),
        ]
        .iter()
        .for_each(|(fen, assumed_steady, expected_steady)| {
            let board = Board::from_str(fen).expect("Valid Position");
            let assumed_steady = bitboard_of_squares(assumed_steady);
            assert_eq!(
                steady_pieces(&board, &assumed_steady),
                bitboard_of_squares(expected_steady) | assumed_steady
            );
        })
    }
}
