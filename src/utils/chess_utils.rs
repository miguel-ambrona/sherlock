//! Util functions.

use chess::{
    get_bishop_rays, get_file, get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_quiets,
    get_rank, get_rook_rays, BitBoard, Board, Color, Piece, Square, EMPTY,
};

use super::LIGHT_SQUARES;

/// Construct a `BitBoard` out of the given squares.
#[cfg(test)]
pub(crate) fn bitboard_of_squares(squares: &[Square]) -> BitBoard {
    squares
        .iter()
        .fold(EMPTY, |acc, s| acc | BitBoard::from_square(*s))
}

/// A `BitBoard` with the pawns of the given color.
#[inline]
pub fn pawns(board: &Board, color: Color) -> BitBoard {
    board.pieces(Piece::Pawn) & board.color_combined(color)
}

/// A `BitBoard` with the knights of the given color.
#[inline]
pub fn knights(board: &Board, color: Color) -> BitBoard {
    board.pieces(Piece::Knight) & board.color_combined(color)
}

/// A `BitBoard` with the bishops of the given color.
#[inline]
pub fn bishops(board: &Board, color: Color) -> BitBoard {
    board.pieces(Piece::Bishop) & board.color_combined(color)
}

/// A `BitBoard` with the rooks of the given color.
#[inline]
pub fn rooks(board: &Board, color: Color) -> BitBoard {
    board.pieces(Piece::Rook) & board.color_combined(color)
}

/// A `BitBoard` with the queens of the given color.
#[inline]
pub fn queens(board: &Board, color: Color) -> BitBoard {
    board.pieces(Piece::Queen) & board.color_combined(color)
}

#[inline]
pub fn square_color(square: Square) -> Color {
    match BitBoard::from_square(square) & LIGHT_SQUARES {
        EMPTY => Color::Black,
        _ => Color::White,
    }
}

pub fn prom_index(piece: Piece) -> usize {
    match piece {
        Piece::Queen => 0,
        Piece::Knight => 1,
        Piece::Rook => 2,
        Piece::Bishop => 3,
        _ => panic!("King or Pawn are not valid promotion types"),
    }
}

/// A `BitBoard` with the squares from which a piece of the given `Piece` type
/// and `Color` can move to from the given `Square` on an empty board.
#[inline]
pub fn moves_on_empty_board(piece: Piece, color: Color, square: Square) -> BitBoard {
    match piece {
        Piece::King => get_king_moves(square),
        Piece::Queen => get_rook_rays(square) | get_bishop_rays(square),
        Piece::Rook => get_rook_rays(square),
        Piece::Bishop => get_bishop_rays(square),
        Piece::Knight => get_knight_moves(square),
        Piece::Pawn => get_pawn_quiets(square, color, EMPTY),
    }
}

/// A `BitBoard` with the squares from which a piece of the given `Piece` type
/// and `Color` can *immediately* reach the given `Square`. By "immediately"
/// we refer to squares at king-distance 1 (except for knight moves).
#[inline]
pub fn predecessors(piece: Piece, color: Color, square: Square) -> BitBoard {
    // Negate the color to get pawn predecessors right.
    let mut predecessors = moves_on_empty_board(piece, !color, square);
    if piece == Piece::Pawn {
        predecessors |= get_pawn_attacks(square, !color, !EMPTY);
        predecessors &= !get_rank(color.to_my_backrank());
    }
    predecessors & (get_king_moves(square) | get_knight_moves(square))
}

/// A `BitBoard` with the squares from which a piece of the given `Piece` type
/// and `Color` will always check an opponent king on the given `Square`,
/// independently of the configuration of other pieces.
#[inline]
pub fn checking_predecessors(piece: Piece, color: Color, square: Square) -> BitBoard {
    let mut predecessors = predecessors(piece, color, square);
    // Remove quiet pawn moves
    if piece == Piece::Pawn {
        predecessors &= !get_file(square.get_file());
    }
    predecessors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;

    #[test]
    fn test_predecessors() {
        [
            (Piece::King, Color::White, A8, vec![A7, B7, B8]),
            (Piece::Queen, Color::Black, D8, vec![C8, C7, D7, E7, E8]),
            (Piece::Rook, Color::White, H1, vec![G1, H2]),
            (Piece::Bishop, Color::White, E2, vec![D1, D3, F1, F3]),
            (Piece::Knight, Color::Black, B1, vec![A3, C3, D2]),
            (Piece::Pawn, Color::White, E7, vec![D6, E6, F6]),
            (Piece::Pawn, Color::Black, E7, vec![]),
            (Piece::Pawn, Color::White, H1, vec![]),
            (Piece::Pawn, Color::Black, H1, vec![H2, G2]),
            (Piece::Pawn, Color::White, E4, vec![D3, E3, F3]),
            (Piece::Pawn, Color::Black, E4, vec![D5, E5, F5]),
        ]
        .into_iter()
        .for_each(|(piece, color, square, expected)| {
            assert_eq!(
                predecessors(piece, color, square),
                bitboard_of_squares(&expected)
            )
        })
    }
}
