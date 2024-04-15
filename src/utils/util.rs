//! Util functions.

use chess::{
    get_bishop_moves, get_king_moves, get_knight_moves, get_pawn_moves, get_rank, get_rook_moves,
    BitBoard, Board, Color, Piece, Square, EMPTY,
};

/// Construct a `BitBoard` out of the given squares.
#[inline]
pub fn bitboard_of_squares(squares: &[Square]) -> BitBoard {
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

/// A `BitBoard` with the squares from which a piece of the given `Piece` type
/// and `Color` can *immediately* reach the given `Square`. By "immediately"
/// we refer to squares at king-distance 1 (except for knight moves).
#[inline]
pub fn predecessors(piece: Piece, color: Color, square: Square) -> BitBoard {
    match piece {
        Piece::King => get_king_moves(square),
        Piece::Queen => get_king_moves(square),
        Piece::Rook => get_rook_moves(square, EMPTY) & get_king_moves(square),
        Piece::Bishop => get_bishop_moves(square, EMPTY) & get_king_moves(square),
        Piece::Knight => get_knight_moves(square),
        Piece::Pawn => get_pawn_moves(square, !color, EMPTY) & !get_rank(color.to_my_backrank()),
    }
}
