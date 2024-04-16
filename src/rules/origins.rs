//! Origins rule.

use chess::{BitBoard, Board, Color, Piece, EMPTY};

use super::{Rule, State};

/// A simple rule that refines the set of origins based on the initial
/// conditions of chess. Queens, rooks, bishops and knights may also come from
/// their relative 2nd rank, as they may be promoted.
///
/// This rule depends solely on the steady pieces, so we keep track of the state
/// of steady pieces the last time this rule was applied to see if we should
/// apply it again.
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

const KING_ORIGINS: [BitBoard; 2] = [BitBoard(16), BitBoard(1152921504606846976)];
const ROOK_ORIGINS: [BitBoard; 2] = [BitBoard(65409), BitBoard(9367205749953921024)];
const PAWN_ORIGINS: [BitBoard; 2] = [BitBoard(65280), BitBoard(71776119061217280)];
const QUEEN_ORIGINS: [BitBoard; 2] = [BitBoard(65288), BitBoard(648236871364640768)];
const BISHOP_ORIGINS: [BitBoard; 2] = [BitBoard(65316), BitBoard(2665849504426622976)];
const KNIGHT_ORIGINS: [BitBoard; 2] = [BitBoard(65346), BitBoard(4827577325564461056)];

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
    use chess::{get_rank, Rank};

    use crate::utils::{
        bitboard_of_squares, A1, A8, B1, B8, C1, C8, D1, D8, E1, E8, F1, F8, G1, G8, H1, H8,
    };

    use super::*;

    #[test]
    fn test_piece_origins() {
        assert_eq!(
            piece_origins(Piece::King, Color::White),
            BitBoard::from_square(E1)
        );
        assert_eq!(
            piece_origins(Piece::Queen, Color::White),
            get_rank(Rank::Second) | BitBoard::from_square(D1)
        );
        assert_eq!(
            piece_origins(Piece::Rook, Color::White),
            get_rank(Rank::Second) | bitboard_of_squares(&[A1, H1])
        );
        assert_eq!(
            piece_origins(Piece::Bishop, Color::White),
            get_rank(Rank::Second) | bitboard_of_squares(&[C1, F1])
        );
        assert_eq!(
            piece_origins(Piece::Knight, Color::White),
            get_rank(Rank::Second) | bitboard_of_squares(&[B1, G1])
        );
        assert_eq!(
            piece_origins(Piece::Pawn, Color::White),
            get_rank(Rank::Second)
        );
        assert_eq!(
            piece_origins(Piece::King, Color::Black),
            BitBoard::from_square(E8)
        );
        assert_eq!(
            piece_origins(Piece::Queen, Color::Black),
            get_rank(Rank::Seventh) | BitBoard::from_square(D8)
        );
        assert_eq!(
            piece_origins(Piece::Rook, Color::Black),
            get_rank(Rank::Seventh) | bitboard_of_squares(&[A8, H8])
        );
        assert_eq!(
            piece_origins(Piece::Bishop, Color::Black),
            get_rank(Rank::Seventh) | bitboard_of_squares(&[C8, F8])
        );
        assert_eq!(
            piece_origins(Piece::Knight, Color::Black),
            get_rank(Rank::Seventh) | bitboard_of_squares(&[B8, G8])
        );
        assert_eq!(
            piece_origins(Piece::Pawn, Color::Black),
            get_rank(Rank::Seventh)
        );
    }
}
