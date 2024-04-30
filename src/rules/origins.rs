//! Origins rule.
//!
//! A simple rule that refines the set of origins based on the initial
//! position of chess.
//! Queens, rooks, bishops and knights may also come from their relative 2nd
//! rank, as they may be promoted.

use chess::{BitBoard, Board, Piece, Square, EMPTY};

use super::{Rule, State};
use crate::utils::square_color;

// This rule depends solely on the steady pieces, so we keep track of the state
// of steady pieces the last time this rule was applied to see if we should
// apply it again.
#[derive(Debug)]
pub struct OriginsRule {
    steady: BitBoard,
}

impl Rule for OriginsRule {
    fn new(_board: &Board) -> Self {
        OriginsRule { steady: EMPTY }
    }

    fn is_applicable(&self, state: &State) -> bool {
        self.steady != state.get_steady() || self.steady == EMPTY
    }

    fn apply(&mut self, state: &mut State) {
        let mut progress = false;

        for square in *state.board.combined() & state.get_steady() & !self.steady {
            let square_origins = BitBoard::from_square(square);
            progress |= state.update_origins(square, square_origins);
        }

        for square in *state.board.combined() & !state.get_steady() {
            let square_origins = state.origins(square)
                & !state.get_steady()
                & COLOR_ORIGINS[state.piece_color_on(square).to_index()]
                & origins_of_piece_on(state.piece_type_on(square), square);
            progress |= state.update_origins(square, square_origins);
        }

        // update the rule state
        self.steady = state.get_steady();

        // report any progress
        state.origins.increase_counter(progress);
        state.progress |= progress;
    }
}

/// The candidate squares from which a piece of the given type which is
/// currently on the given square may have started the game.
#[inline]
fn origins_of_piece_on(piece: Piece, square: Square) -> BitBoard {
    match piece {
        Piece::King => KING_ORIGINS,
        Piece::Queen => QUEEN_ORIGINS,
        Piece::Rook => ROOK_ORIGINS,
        Piece::Bishop => BISHOP_ORIGINS[square_color(square).to_index()],
        Piece::Knight => KNIGHT_ORIGINS,
        Piece::Pawn => PAWN_ORIGINS[square.to_index()],
    }
}

pub const COLOR_ORIGINS: [BitBoard; 2] = [
    BitBoard(65535),                // 1st & 2nd ranks
    BitBoard(18446462598732840960), // 7th & 8th ranks
];
const KING_ORIGINS: BitBoard = BitBoard(1152921504606846992); // E1, E8
const QUEEN_ORIGINS: BitBoard = BitBoard(648236871364706056); // D1, D8, 2nd & 7th ranks
const ROOK_ORIGINS: BitBoard = BitBoard(9367205749953986433); // A1, H1, A8, H8, 2nd & 7th ranks
const KNIGHT_ORIGINS: BitBoard = BitBoard(4827577325564526402); // B1, G1, B8, G8, 2nd & 7th ranks
const BISHOP_ORIGINS: [BitBoard; 2] = [
    BitBoard(360006495212994336),  // F1, C8, 2nd & 7th ranks
    BitBoard(2377619128274976516), // C1, F8, 2nd & 7th ranks
];
const PAWN_ORIGINS: [BitBoard; 64] = [
    BitBoard(0),                 // A1: N/A
    BitBoard(0),                 // B1: N/A
    BitBoard(0),                 // C1: N/A
    BitBoard(0),                 // D1: N/A
    BitBoard(0),                 // E1: N/A
    BitBoard(0),                 // F1: N/A
    BitBoard(0),                 // G1: N/A
    BitBoard(0),                 // H1: N/A
    BitBoard(17732923532771584), // A2: A2, A7, B7, C7, D7, E7, F7
    BitBoard(35747322042253824), // B2: B2, A7, B7, C7, D7, E7, F7, G7
    BitBoard(71776119061218304), // C2: C2, A7, B7, C7, D7, E7, F7, G7, H7
    BitBoard(71776119061219328), // D2: D2, A7, B7, C7, D7, E7, F7, G7, H7
    BitBoard(71776119061221376), // E2: E2, A7, B7, C7, D7, E7, F7, G7, H7
    BitBoard(71776119061225472), // F2: F2, A7, B7, C7, D7, E7, F7, G7, H7
    BitBoard(71494644084523008), // G2: G2, B7, C7, D7, E7, F7, G7, H7
    BitBoard(70931694131118080), // H2: H2, C7, D7, E7, F7, G7, H7
    BitBoard(8725724278031104),  // A3: A2, B2, A7, B7, C7, D7, E7
    BitBoard(17732923532773120), // B3: A2, B2, C2, A7, B7, C7, D7, E7, F7
    BitBoard(35747322042256896), // C3: B2, C2, D2, A7, B7, C7, D7, E7, F7, G7
    BitBoard(71776119061224448), // D3: C2, D2, E2, A7, B7, C7, D7, E7, F7, G7, H7
    BitBoard(71776119061231616), // E3: D2, E2, F2, A7, B7, C7, D7, E7, F7, G7, H7
    BitBoard(71494644084535296), // F3: E2, F2, G2, B7, C7, D7, E7, F7, G7, H7
    BitBoard(70931694131142656), // G3: F2, G2, H2, C7, D7, E7, F7, G7, H7
    BitBoard(69805794224291840), // H3: G2, H2, D7, E7, F7, G7, H7
    BitBoard(4222124650661632),  // A4: A2, B2, C2, A7, B7, C7, D7
    BitBoard(8725724278034176),  // B4: A2, B2, C2, D2, A7, B7, C7, D7, E7
    BitBoard(17732923532779264), // C4: A2, B2, C2, D2, E2, A7, B7, C7, D7, E7, F7
    BitBoard(35747322042269184), // D4: B2, C2, D2, E2, F2, A7, B7, C7, D7, E7, F7, G7
    BitBoard(71494644084538368), // E4: C2, D2, E2, F2, G2, B7, C7, D7, E7, F7, G7, H7
    BitBoard(70931694131148800), // F4: D2, E2, F2, G2, H2, C7, D7, E7, F7, G7, H7
    BitBoard(69805794224304128), // G4: E2, F2, G2, H2, D7, E7, F7, G7, H7
    BitBoard(67553994410614784), // H4: F2, G2, H2, E7, F7, G7, H7
    BitBoard(1970324836978432),  // A5: A2, B2, C2, D2, A7, B7, C7
    BitBoard(4222124650667776),  // B5: A2, B2, C2, D2, E2, A7, B7, C7, D7
    BitBoard(8725724278046464),  // C5: A2, B2, C2, D2, E2, F2, A7, B7, C7, D7, E7
    BitBoard(17451448556093184), // D5: A2, B2, C2, D2, E2, F2, G2, B7, C7, D7, E7, F7
    BitBoard(34902897112186368), // E5: B2, C2, D2, E2, F2, G2, H2, C7, D7, E7, F7, G7
    BitBoard(69805794224307200), // F5: C2, D2, E2, F2, G2, H2, D7, E7, F7, G7, H7
    BitBoard(67553994410620928), // G5: D2, E2, F2, G2, H2, E7, F7, G7, H7
    BitBoard(63050394783248384), // H5: E2, F2, G2, H2, F7, G7, H7
    BitBoard(844424930139904),   // A6: A2, B2, C2, D2, E2, A7, B7
    BitBoard(1970324836990720),  // B6: A2, B2, C2, D2, E2, F2, A7, B7, C7
    BitBoard(3940649673981696),  // C6: A2, B2, C2, D2, E2, F2, G2, B7, C7, D7
    BitBoard(7881299347963648),  // D6: A2, B2, C2, D2, E2, F2, G2, H2, C7, D7, E7
    BitBoard(15762598695862016), // E6: A2, B2, C2, D2, E2, F2, G2, H2, D7, E7, F7
    BitBoard(31525197391658496), // F6: B2, C2, D2, E2, F2, G2, H2, E7, F7, G7
    BitBoard(63050394783251456), // G6: C2, D2, E2, F2, G2, H2, F7, G7, H7
    BitBoard(54043195528509440), // H6: D2, E2, F2, G2, H2, G7, H7
    BitBoard(281474976726784),   // A7: A2, B2, C2, D2, E2, F2, A7
    BitBoard(562949953453824),   // B7: A2, B2, C2, D2, E2, F2, G2, B7
    BitBoard(1125899906907904),  // C7: A2, B2, C2, D2, E2, F2, G2, H2, C7
    BitBoard(2251799813750528),  // D7: A2, B2, C2, D2, E2, F2, G2, H2, D7
    BitBoard(4503599627435776),  // E7: A2, B2, C2, D2, E2, F2, G2, H2, E7
    BitBoard(9007199254806272),  // F7: A2, B2, C2, D2, E2, F2, G2, H2, F7
    BitBoard(18014398509547008), // G7: B2, C2, D2, E2, F2, G2, H2, G7
    BitBoard(36028797019028480), // H7: C2, D2, E2, F2, G2, H2, H7
    BitBoard(0),                 // A8: N/A
    BitBoard(0),                 // B8: N/A
    BitBoard(0),                 // C8: N/A
    BitBoard(0),                 // D8: N/A
    BitBoard(0),                 // E8: N/A
    BitBoard(0),                 // F8: N/A
    BitBoard(0),                 // G8: N/A
    BitBoard(0),                 // H8: N/A
];

#[cfg(test)]
mod tests {
    use chess::{get_rank, Rank};

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_origins_of_piece_on() {
        let pawn_ranks = get_rank(Rank::Second) | get_rank(Rank::Seventh);
        assert_eq!(
            origins_of_piece_on(Piece::King, A1),
            bitboard_of_squares(&[E1, E8])
        );
        assert_eq!(
            origins_of_piece_on(Piece::Queen, D1),
            bitboard_of_squares(&[D1, D8]) | pawn_ranks
        );
        assert_eq!(
            origins_of_piece_on(Piece::Rook, B1),
            bitboard_of_squares(&[A1, H1, A8, H8]) | pawn_ranks
        );
        assert_eq!(
            origins_of_piece_on(Piece::Knight, H8),
            bitboard_of_squares(&[B1, G1, B8, G8]) | pawn_ranks
        );
        assert_eq!(
            origins_of_piece_on(Piece::Bishop, B1),
            bitboard_of_squares(&[F1, C8]) | pawn_ranks
        );
        assert_eq!(
            origins_of_piece_on(Piece::Bishop, A1),
            bitboard_of_squares(&[C1, F8]) | pawn_ranks
        );
        assert_eq!(
            origins_of_piece_on(Piece::Pawn, H4),
            bitboard_of_squares(&[F2, G2, H2, E7, F7, G7, H7])
        );
        assert_eq!(
            origins_of_piece_on(Piece::Pawn, C6),
            bitboard_of_squares(&[A2, B2, C2, D2, E2, F2, G2, B7, C7, D7])
        );
    }
}
