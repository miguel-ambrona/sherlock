//! Corner knight rule.
//!
//! This rule explits the fact that if there exist black pawns on B6, B7, C7,
//! any knight promoted on A8 cannot have possibly left its promotion square.
//! Furthermore, no piece can possibly have reached B6 from promotion on A8.
//!
//! A similar argument extends to all other corners.

use chess::{BitBoard, Color, Piece, PROMOTION_PIECES};

use super::{Analysis, Rule};
use crate::utils::{A1, A8, H1, H8};

const B3_B2_C2: BitBoard = BitBoard(132608);
const G3_G2_F2: BitBoard = BitBoard(4218880);
const B6_B7_C7: BitBoard = BitBoard(1691048883519488);
const G6_G7_F7: BitBoard = BitBoard(27091966508400640);

#[derive(Debug)]
pub struct CornerKnightRule {
    applied: bool,
}

impl Rule for CornerKnightRule {
    fn new() -> Self {
        CornerKnightRule { applied: false }
    }

    fn update(&mut self, _analysis: &Analysis) {
        self.applied = true;
    }

    fn is_applicable(&self, _analysis: &Analysis) -> bool {
        !self.applied
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for (color, trinity, corner_square) in [
            (Color::White, B3_B2_C2, A1),
            (Color::White, G3_G2_F2, H1),
            (Color::Black, B6_B7_C7, A8),
            (Color::Black, G6_G7_F7, H8),
        ] {
            let pawns = analysis.board.pieces(Piece::Pawn);
            let color_pieces = analysis.board.color_combined(color);

            // all three trinity squares contain pawns of the desired color
            if pawns & color_pieces & trinity == trinity {
                // a knight promoted on the corner cannot go anywhere
                progress |= analysis.update_reachable_from_promotion(
                    !color,
                    Piece::Knight,
                    corner_square.get_file(),
                    BitBoard::from_square(corner_square),
                );

                // no piece promoted on the corner can go to the trinity squares
                for prom_piece in PROMOTION_PIECES {
                    progress |= analysis.update_reachable_from_promotion(
                        !color,
                        prom_piece,
                        corner_square.get_file(),
                        !trinity,
                    )
                }
            }
        }

        progress
    }
}
