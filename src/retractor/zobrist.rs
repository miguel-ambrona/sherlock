use chess::{CastleRights, Color, File, Piece, Square};

// Include the generated lookup tables
include!(concat!(env!("OUT_DIR"), "/zobrist_gen.rs"));

// use super::zobrist_gen::{SIDE_TO_MOVE, ZOBRIST_CASTLES, ZOBRIST_EP,
// ZOBRIST_PIECES};

/// Create a completely blank type.  This allows all the functions to be part of
/// this type, which I think is a bit cleaner than bare functions everywhere.
pub struct Zobrist;

impl Zobrist {
    #[inline]
    pub const fn color() -> u64 {
        SIDE_TO_MOVE
    }

    #[inline]
    pub fn piece(piece: Piece, square: Square, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_PIECES
                .get_unchecked(color.to_index())
                .get_unchecked(piece.to_index())
                .get_unchecked(square.to_index())
        }
    }

    #[inline]
    pub fn castles(castle_rights: CastleRights, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_CASTLES
                .get_unchecked(color.to_index())
                .get_unchecked(castle_rights.to_index())
        }
    }

    #[inline]
    pub fn en_passant(file: File, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_EP
                .get_unchecked(color.to_index())
                .get_unchecked(file.to_index())
        }
    }

    #[inline]
    pub const fn ep_any() -> u64 {
        ZOBRIST_EP_ANY
    }
}
