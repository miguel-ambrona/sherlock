//! Origins rule.

use chess::{get_rank, BitBoard, Board, Color, Piece, Rank, EMPTY};
use lazy_static::lazy_static;

use super::{Rule, State};
use crate::utils::{
    bitboard_of_squares, A1, A8, B1, B8, C1, C8, D1, D8, E1, E8, F1, F8, G1, G8, H1, H8,
};

/// A simple rule that refines the set of origins based on the initial
/// conditions of chess. Queens, rooks, bishops and knights may also come from
/// their relative 2nd rank, as they may be promoted.
///
/// This rule depends solely on the steady pieces, so we keep track of the state
/// of steady pieces the last time this rule was applied to see if we should apply it
/// again.
#[derive(Debug)]
pub struct OriginsRule {
    steady: BitBoard,
}

impl Rule for OriginsRule {
    fn new(_board: &Board) -> Self {
        OriginsRule { steady: EMPTY }
    }

    fn is_applicable(&self, state: &State) -> bool {
        self.steady != state.steady || self.steady == EMPTY
    }

    fn apply(&mut self, state: &mut State) {
        let mut progress = false;

        for square in *state.board.combined() & state.steady & !self.steady {
            let square_origins = BitBoard::from_square(square);
            progress = progress || (state.origins(square) != square_origins);
            state.origins.0[square.to_index()] = square_origins;
        }

        for square in *state.board.combined() & !state.steady {
            let square_origins = state.origins(square)
                & !state.steady
                & piece_origins(state.piece_type_on(square), state.piece_color_on(square));
            progress = progress || (state.origins(square) != square_origins);
            state.origins.0[square.to_index()] = square_origins;
        }

        // update the rule state and report any progress
        self.steady = state.steady;
        if progress {
            state.origins.1 += 1;
            state.progress = true;
        }
    }
}

lazy_static! {
    static ref KING_ORIGINS: [BitBoard; 2] = [BitBoard::from_square(E1), BitBoard::from_square(E8)];
    static ref QUEEN_ORIGINS: [BitBoard; 2] = [
        BitBoard::from_square(D1) | get_rank(Rank::Second),
        BitBoard::from_square(D8) | get_rank(Rank::Seventh)
    ];
    static ref ROOK_ORIGINS: [BitBoard; 2] = [
        bitboard_of_squares(&[A1, H1]) | get_rank(Rank::Second),
        bitboard_of_squares(&[A8, H8]) | get_rank(Rank::Seventh)
    ];
    static ref BISHOP_ORIGINS: [BitBoard; 2] = [
        bitboard_of_squares(&[C1, F1]) | get_rank(Rank::Second),
        bitboard_of_squares(&[C8, F8]) | get_rank(Rank::Seventh)
    ];
    static ref KNIGHT_ORIGINS: [BitBoard; 2] = [
        bitboard_of_squares(&[B1, G1]) | get_rank(Rank::Second),
        bitboard_of_squares(&[B8, G8]) | get_rank(Rank::Seventh)
    ];
    static ref PAWN_ORIGINS: [BitBoard; 2] = [get_rank(Rank::Second), get_rank(Rank::Seventh)];
}

/// The candidate squares from which a piece of the given type and color may
/// have started the game.
#[inline]
fn piece_origins(piece: Piece, color: Color) -> BitBoard {
    match piece {
        Piece::King => KING_ORIGINS[color.to_index()],
        Piece::Queen => QUEEN_ORIGINS[color.to_index()],
        Piece::Rook => ROOK_ORIGINS[color.to_index()],
        Piece::Bishop => BISHOP_ORIGINS[color.to_index()],
        Piece::Knight => KNIGHT_ORIGINS[color.to_index()],
        Piece::Pawn => PAWN_ORIGINS[color.to_index()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{A2, B2, C2, D2, E2, F2, G2, H2};

    #[test]
    fn test_piece_origins() {
        assert_eq!(
            piece_origins(Piece::Pawn, Color::White),
            bitboard_of_squares(&[A2, B2, C2, D2, E2, F2, G2, H2])
        );
        assert_eq!(
            piece_origins(Piece::Knight, Color::White),
            get_rank(Rank::Second) | bitboard_of_squares(&[B1, G1])
        );
        assert_eq!(
            piece_origins(Piece::King, Color::Black),
            BitBoard::from_square(E8)
        );
        assert_eq!(
            piece_origins(Piece::Bishop, Color::Black),
            get_rank(Rank::Seventh) | bitboard_of_squares(&[C8, F8])
        );
    }
}
