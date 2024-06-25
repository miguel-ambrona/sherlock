use std::{
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

use chess::{
    between, get_bishop_rays, get_knight_moves, get_pawn_attacks, get_rook_rays, BitBoard, Board,
    CastleRights, Color, File, Piece, Rank, Square, ALL_FILES, ALL_RANKS, EMPTY, NUM_COLORS,
    NUM_PIECES,
};

use super::{chess_retraction::ChessRetraction, zobrist::Zobrist};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum EnPassantFlag {
    Any,
    Some(Square),
    None,
}

/// A representation of a retractable chess board.
///
/// Unlike a normal board, the en-passant information after a retraction may be
/// uncertain, we allow the en-passant flag to take three forms:
///  - Any
///  - Some(Square)
///  - None
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RetractableBoard {
    pieces: [BitBoard; NUM_PIECES],
    color_combined: [BitBoard; NUM_COLORS],
    combined: BitBoard,
    side_to_move: Color,
    castle_rights: [CastleRights; NUM_COLORS],
    pinned: BitBoard,
    checkers: BitBoard,
    hash: u64,
    en_passant: EnPassantFlag,
}

impl From<Board> for RetractableBoard {
    fn from(board: Board) -> Self {
        Self {
            pieces: [
                *board.pieces(Piece::Pawn),
                *board.pieces(Piece::Knight),
                *board.pieces(Piece::Bishop),
                *board.pieces(Piece::Rook),
                *board.pieces(Piece::Queen),
                *board.pieces(Piece::King),
            ],
            color_combined: [
                *board.color_combined(Color::White),
                *board.color_combined(Color::Black),
            ],
            combined: *board.combined(),
            side_to_move: board.side_to_move(),
            castle_rights: [
                board.castle_rights(Color::White),
                board.castle_rights(Color::Black),
            ],
            pinned: *board.pinned(),
            checkers: *board.checkers(),
            hash: board.get_hash(),
            en_passant: match board.en_passant() {
                Some(ep_square) => EnPassantFlag::Some(ep_square),
                None => EnPassantFlag::None,
            },
        }
    }
}

impl Default for RetractableBoard {
    fn default() -> Self {
        Board::default().into()
    }
}

impl Hash for RetractableBoard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl EnPassantFlag {
    fn is_some(&self) -> bool {
        match self {
            EnPassantFlag::Some(_) => true,
            EnPassantFlag::Any => false,
            EnPassantFlag::None => false,
        }
    }

    fn zobrist(&self, side_to_move: Color) -> u64 {
        match self {
            EnPassantFlag::Some(ep_square) => {
                Zobrist::en_passant(ep_square.get_file(), !side_to_move)
            }
            EnPassantFlag::Any => Zobrist::ep_any(),
            EnPassantFlag::None => 0,
        }
    }
}

impl fmt::Display for RetractableBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut count = 0;
        for rank in ALL_RANKS.iter().rev() {
            for file in ALL_FILES.iter() {
                let square = Square::make_square(*rank, *file);

                if self.piece_on(square).is_some() && count != 0 {
                    write!(f, "{}", count)?;
                    count = 0;
                }

                if let Some(piece) = self.piece_on(square) {
                    let color = if BitBoard::from_square(square) & self.color_combined(Color::White)
                        != EMPTY
                    {
                        Color::White
                    } else {
                        Color::Black
                    };
                    write!(f, "{}", piece.to_string(color))?;
                } else {
                    count += 1;
                }
            }

            if count != 0 {
                write!(f, "{}", count)?;
            }

            if *rank != Rank::First {
                write!(f, "/")?;
            }
            count = 0;
        }

        write!(f, " ")?;

        if self.side_to_move == Color::White {
            write!(f, "w ")?;
        } else {
            write!(f, "b ")?;
        }

        write!(
            f,
            "{}",
            self.castle_rights[Color::White.to_index()].to_string(Color::White)
        )?;
        write!(
            f,
            "{}",
            self.castle_rights[Color::Black.to_index()].to_string(Color::Black)
        )?;
        if self.castle_rights[0] == CastleRights::NoRights
            && self.castle_rights[1] == CastleRights::NoRights
        {
            write!(f, "-")?;
        }

        write!(f, " ")?;
        if let EnPassantFlag::Some(sq) = self.en_passant {
            write!(f, "{}", sq)?;
        } else if self.en_passant == EnPassantFlag::None {
            write!(f, "-")?;
        } else {
            write!(f, "?")?;
        }

        write!(f, "")
    }
}

impl RetractableBoard {
    /// Create a `RetractableBoard` from a FEN string.
    pub fn from_fen(fen: &str) -> Result<RetractableBoard, chess::Error> {
        Board::from_str(fen).map(|board| board.into())
    }

    /// A `BitBoard` with all the pieces of the given type (and both colors).
    pub fn pieces(&self, piece: Piece) -> &BitBoard {
        unsafe { self.pieces.get_unchecked(piece.to_index()) }
    }

    /// What piece is on a particular `Square`?  Is there even one?
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        let square_bb = BitBoard::from_square(square);
        if self.combined() & square_bb == EMPTY {
            None
        } else if (self.pieces(Piece::Pawn)
            ^ self.pieces(Piece::Knight)
            ^ self.pieces(Piece::Bishop))
            & square_bb
            != EMPTY
        {
            if self.pieces(Piece::Pawn) & square_bb != EMPTY {
                Some(Piece::Pawn)
            } else if self.pieces(Piece::Knight) & square_bb != EMPTY {
                Some(Piece::Knight)
            } else {
                Some(Piece::Bishop)
            }
        } else if self.pieces(Piece::Rook) & square_bb != EMPTY {
            Some(Piece::Rook)
        } else if self.pieces(Piece::Queen) & square_bb != EMPTY {
            Some(Piece::Queen)
        } else {
            Some(Piece::King)
        }
    }

    /// Who's turn is it?
    #[inline]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// A `BitBoard` with all the pieces.
    #[inline]
    pub fn combined(&self) -> &BitBoard {
        &self.combined
    }

    /// A `BitBoard` with all the pieces of the given color.
    #[inline]
    pub fn color_combined(&self, color: Color) -> &BitBoard {
        unsafe { self.color_combined.get_unchecked(color.to_index()) }
    }

    /// The en_passant flag.
    pub(crate) fn en_passant(self) -> EnPassantFlag {
        self.en_passant
    }

    /// The `CastleRights` of the given `Color`.
    #[inline]
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        unsafe { *self.castle_rights.get_unchecked(color.to_index()) }
    }

    /// The `BitBoard` of pinned pieces.
    #[inline]
    pub fn pinned(&self) -> &BitBoard {
        &self.pinned
    }

    /// The `Bitboard` of current checkers.
    #[inline]
    pub fn checkers(&self) -> &BitBoard {
        &self.checkers
    }

    /// The `Square` where it king of the given `Color` currently is.
    #[inline]
    pub fn king_square(&self, color: Color) -> Square {
        (self.pieces(Piece::King) & self.color_combined(color)).to_square()
    }

    /// Specify that the en-passant information is uncertain, this will only
    /// have an effect if the en-passant flag is currently set to `None`.
    #[inline]
    pub fn set_uncertain_ep(&mut self) {
        if self.en_passant == EnPassantFlag::None {
            self.hash ^= Zobrist::ep_any();
            self.en_passant = EnPassantFlag::Any;
        }
    }

    /// Add or remove a piece from the bitboards in this struct.
    fn xor(&mut self, piece: Piece, bb: BitBoard, color: Color) {
        unsafe {
            *self.pieces.get_unchecked_mut(piece.to_index()) ^= bb;
            *self.color_combined.get_unchecked_mut(color.to_index()) ^= bb;
            self.combined ^= bb;
            for square in bb {
                self.hash ^= Zobrist::piece(piece, square, color);
            }
        }
    }

    /// Apply a chess retraction to the given board, creating a new board.
    #[inline]
    pub fn make_retraction_new(&self, r: ChessRetraction) -> RetractableBoard {
        let mut result = *self;
        result.side_to_move = !self.side_to_move;
        result.en_passant = EnPassantFlag::Any;
        result.checkers = EMPTY;
        result.pinned = EMPTY;
        let source = r.source();
        let target = r.target();
        let side_to_retract = result.side_to_move;

        let source_bb = BitBoard::from_square(source);
        let target_bb = BitBoard::from_square(target);
        let retraction_bb = source_bb ^ target_bb;
        let retracted_piece = self.piece_on(source).unwrap();

        if r.unpromotion() {
            result.xor(retracted_piece, source_bb, side_to_retract);
            result.xor(Piece::Pawn, target_bb, side_to_retract);
        } else {
            result.xor(retracted_piece, retraction_bb, side_to_retract);
        }
        if let Some(uncaptured_piece) = r.uncaptured() {
            result.xor(uncaptured_piece, source_bb, self.side_to_move);
        }

        // handle uncastling when applicable
        const CASTLE_MOVES: BitBoard = BitBoard(6052837899185946708);
        if retracted_piece == Piece::King && (retraction_bb & CASTLE_MOVES) == retraction_bb {
            debug_assert!(self.castle_rights[side_to_retract.to_index()] == CastleRights::NoRights);

            let my_backrank = side_to_retract.to_my_backrank();
            let (rook_files, recovered_rights) = match source.get_file() {
                File::G => ((File::F, File::H), CastleRights::KingSide),
                File::C => ((File::D, File::A), CastleRights::QueenSide),
                _ => unreachable!(),
            };
            let rook_bb =
                BitBoard::set(my_backrank, rook_files.0) ^ BitBoard::set(my_backrank, rook_files.1);
            result.xor(Piece::Rook, rook_bb, side_to_retract);
            unsafe {
                *result
                    .castle_rights
                    .get_unchecked_mut(side_to_retract.to_index()) = recovered_rights;
            }

            // update zobrist hash about castling
            result.hash ^= Zobrist::castles(CastleRights::NoRights, side_to_retract)
                ^ Zobrist::castles(recovered_rights, side_to_retract)
        }

        // handle en-passant uncaptures when applicable
        let en_passant_uncapture = retracted_piece == Piece::Pawn
            && r.uncaptured().is_none()
            && source.get_file() != target.get_file();
        if en_passant_uncapture {
            let reappearing_pawn_square = source.ubackward(side_to_retract);
            result.en_passant = EnPassantFlag::Some(reappearing_pawn_square);
            result.xor(
                Piece::Pawn,
                BitBoard::from_square(reappearing_pawn_square),
                self.side_to_move,
            );
        }

        // update zobrist hash about turn
        result.hash ^= Zobrist::color();

        // update zobrist hash about en-passant
        if result.en_passant != self.en_passant || result.en_passant.is_some() {
            result.hash ^= self.en_passant.zobrist(self.side_to_move)
                ^ result.en_passant.zobrist(result.side_to_move);
        }

        let king_bb = result.pieces(Piece::King) & result.color_combined(side_to_retract);
        let king_square = king_bb.to_square();

        // update knight checks
        if retracted_piece == Piece::King || r.uncaptured() == Some(Piece::Knight) {
            let knights = result.pieces(Piece::Knight) & result.color_combined(self.side_to_move);
            result.checkers ^= get_knight_moves(king_square) & knights;
        }

        // update pawn checks
        if retracted_piece == Piece::King
            || r.uncaptured() == Some(Piece::Pawn)
            || en_passant_uncapture
        {
            let pawns = result.pieces(Piece::Pawn) & result.color_combined(self.side_to_move);
            result.checkers ^= get_pawn_attacks(king_square, side_to_retract, pawns);
        }

        // let's update sliding attackers and pins
        let sliding_attackers = {
            let bishops = result.pieces(Piece::Bishop) | result.pieces(Piece::Queen);
            let rooks = result.pieces(Piece::Rook) | result.pieces(Piece::Queen);
            result.color_combined(self.side_to_move)
                & (get_bishop_rays(king_square) & bishops | get_rook_rays(king_square) & rooks)
        };

        for square in sliding_attackers {
            let between = between(square, king_square) & result.combined();
            if between == EMPTY {
                result.checkers ^= BitBoard::from_square(square);
            } else if between.popcnt() == 1 {
                result.pinned ^= between;
            }
        }

        result
    }
}

#[cfg(test)]
use crate::utils::*;

#[test]
fn test_make_retraction_new() {
    [
        (
            "2nR3K/pk1Rp1p1/p2p4/P1p5/1Pp4B/2PP2P1/4P2P/n7 b - -",
            ChessRetraction::new(D8, C7, Some(Piece::Knight), true),
            "2nn3K/pkPRp1p1/p2p4/P1p5/1Pp4B/2PP2P1/4P2P/n7 w - -",
        ),
        (
            "4k3/8/8/7K/8/8/8/8 b - -",
            ChessRetraction::new(H5, G6, Some(Piece::Rook), false),
            "4k3/8/6K1/7r/8/8/8/8 w - -",
        ),
        (
            "5k2/8/8/8/8/8/8/5RK1 b - -",
            ChessRetraction::new(G1, E1, None, false),
            "5k2/8/8/8/8/8/8/4K2R w K -",
        ),
        (
            "r1bq1r2/pp2n3/4N1Pk/3pPp2/1b1n2Q1/2N5/PP3PP1/R1B1K2R b KQ -",
            ChessRetraction::new(G6, H5, None, false),
            "r1bq1r2/pp2n3/4N2k/3pPppP/1b1n2Q1/2N5/PP3PP1/R1B1K2R w KQ g6",
        ),
        (
            "2kr3r/5p2/2p3p1/7Q/B7/4P3/8/K3R3 w - -",
            ChessRetraction::new(C8, E8, None, false),
            "r3k2r/5p2/2p3p1/7Q/B7/4P3/8/K3R3 b q -",
        ),
        (
            "3kr3/8/8/8/8/8/3p4/3K4 b - -",
            ChessRetraction::new(D1, E1, None, false),
            "3kr3/8/8/8/8/8/3p4/4K3 w - -",
        ),
    ]
    .iter()
    .for_each(|(fen, r, expected_fen)| {
        let board: RetractableBoard = Board::from_str(fen).unwrap().into();
        let retracted_board = board.make_retraction_new(*r);

        let ep_correction = match retracted_board.en_passant {
            EnPassantFlag::Any => Zobrist::ep_any(),
            _ => 0,
        };

        assert_eq!(
            retracted_board.hash ^ ep_correction,
            Board::from_str(expected_fen).unwrap().get_hash()
        );
    })
}
