//! Util functions.

use chess::{
    get_bishop_moves, get_bishop_rays, get_file, get_king_moves, get_knight_moves,
    get_pawn_attacks, get_pawn_quiets, get_rank, get_rook_moves, get_rook_rays, BitBoard, Color,
    Piece, Rank, Square, EMPTY,
};

use super::LIGHT_SQUARES;
use crate::RetractableBoard;

/// An array representing all 12 different (colored) pieces, each consisting of
/// a pair including their color and their piece type.
pub const ALL_COLORED_PIECES: [(Color, Piece); 12] = [
    (Color::White, Piece::Pawn),
    (Color::White, Piece::Knight),
    (Color::White, Piece::Bishop),
    (Color::White, Piece::Rook),
    (Color::White, Piece::Queen),
    (Color::White, Piece::King),
    (Color::Black, Piece::Pawn),
    (Color::Black, Piece::Knight),
    (Color::Black, Piece::Bishop),
    (Color::Black, Piece::Rook),
    (Color::Black, Piece::Queen),
    (Color::Black, Piece::King),
];

/// Construct a `BitBoard` out of the given squares.
#[cfg(test)]
pub(crate) fn bitboard_of_squares(squares: &[Square]) -> BitBoard {
    squares
        .iter()
        .fold(EMPTY, |acc, s| acc | BitBoard::from_square(*s))
}

#[inline]
pub fn square_color(square: Square) -> Color {
    match BitBoard::from_square(square) & LIGHT_SQUARES {
        EMPTY => Color::Black,
        _ => Color::White,
    }
}

pub fn origin_color(origin: Square) -> Color {
    match origin.get_rank() {
        Rank::First | Rank::Second => Color::White,
        Rank::Seventh | Rank::Eighth => Color::Black,
        _ => panic!("Not an origin square"),
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

/// Returns `Some piece` iff all the given squares contain a piece of type
/// `piece`. Returns `None` otherwise.
pub fn common_piece_in_all_squares(board: &RetractableBoard, squares: BitBoard) -> Option<Piece> {
    if squares == EMPTY {
        return None;
    }
    let piece_opt = board.piece_on(squares.to_square());
    for square in squares {
        if board.piece_on(square) != piece_opt {
            return None;
        }
    }
    piece_opt
}

/// Returns `true` iff the given square is attacked by the given color in the
/// given board.
pub fn is_attacked(board: &RetractableBoard, square: Square, color: Color) -> bool {
    let combined = board.combined();
    let color_pieces = board.color_combined(color);

    let mut attackers = EMPTY;

    attackers |= get_rook_moves(square, *combined)
        & (board.pieces(Piece::Rook) | board.pieces(Piece::Queen))
        & color_pieces;

    attackers |= get_bishop_moves(square, *combined)
        & (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
        & color_pieces;

    attackers |= get_knight_moves(square) & board.pieces(Piece::Knight) & color_pieces;

    attackers |= get_king_moves(square) & board.pieces(Piece::King) & color_pieces;

    attackers |= get_pawn_attacks(square, !color, board.pieces(Piece::Pawn) & color_pieces);

    attackers != EMPTY
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
