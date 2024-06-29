use std::cmp::max;

use arrayvec::ArrayVec;
use chess::{get_file, get_rank, BitBoard, Piece, Square, ALL_SQUARES, EMPTY};
use nodrop::NoDrop;

use super::{
    chess_retraction::ChessRetraction,
    piece_type::{
        BishopType, InDoubleCheck, InSimpleCheck, KingType, KnightType, NotInCheck, PawnType,
        PieceType, QueenType, RookType,
    },
};
use crate::{
    rules::{origins_of_piece_on, COLOR_ORIGINS},
    utils::{DARK_SQUARES, LIGHT_SQUARES, PROMOTION_RANKS},
    Analysis, EnPassantFlag, RetractableBoard,
};

/// The kind of uncapture of a [SourceAndTargets] object, specifying whether the
/// uncapture is optional, necessary, forbidden or an en-passant uncapture.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum UnCaptureKind {
    /// An uncapture is optimal if the retraction may or may not have uncaptured
    /// an opponent piece. This is the most common type of uncapture.
    Optional,
    /// An uncapture can be necessary, for example if the retracting piece type
    /// is a pawn that retracts to an adjacent file.
    Necessary,
    /// An uncapture can be forbidden, for example when retracting a pawn push
    /// or uncastling.
    Forbidden,
    /// En-passant retractions are specified through this case.
    UnEnPassant,
}

/// A collection of retractions encoding the `source` square from where we are
/// retracting, the `target` squares where we can retract into and additional
/// information about the uncapture kind (whether we can optionally, must or
/// must not uncapture a piece) and whether the retraction is an unpromotion.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct SourceAndTargets {
    source: Square,
    targets: BitBoard,
    uncapture_kind: UnCaptureKind,
    unpromotion: bool,
}

impl SourceAndTargets {
    pub(crate) fn new(
        source: Square,
        targets: BitBoard,
        uncapture_kind: UnCaptureKind,
        unpromotion: bool,
    ) -> SourceAndTargets {
        SourceAndTargets {
            source,
            targets,
            uncapture_kind,
            unpromotion,
        }
    }
}

/// The needed buffer size is often just the number of pieces, 18 at most.
/// However, in the worst-case scenario, when the whole 8th rank is populated
/// with candidate retractors (that may can be unpromoted) and there are 7 pawns
/// on the 6th rank (un-enpassant candidates), the buffer size will need to be:
///  - 15 * 2 for pawn unpushes & pawn uncaptures
///  - 7 for pawn en-passant uncaptures
///  - 8 officer retractions
///  - 2 for king retractions (uncastling is not an option given that all
///    officers are on the 8th rank)
/// which makes a total of 47.
///
/// TODO: Double-check this and reduce it by modifying the `SourceAndTargets`
/// type. We could have several target bitboard depending on the uncapture kind
/// and remove the uncapture kind field.
const BUFFER_SIZE: usize = 47;
pub(crate) type RetractionList = NoDrop<ArrayVec<SourceAndTargets, BUFFER_SIZE>>;

/// How many pieces can be uncaptured?
const NUM_UNCAPTURES: usize = 6;

/// What pieces can be uncaptured?
const UNCAPTURES: [Option<Piece>; NUM_UNCAPTURES] = [
    None,
    Some(Piece::Pawn),
    Some(Piece::Knight),
    Some(Piece::Bishop),
    Some(Piece::Rook),
    Some(Piece::Queen),
];

/// An incremental retractions generator.
///
/// This structure allows us to enumerate all retractions through an iterator
/// that can meet a certain pattern, e.g. avoiding retractions into a certain
/// mask of targets; or avoding retractions that uncapture certain piece types
/// on certain squares.
pub struct RetractionGen {
    retractions: RetractionList,
    index: usize,
    targets_mask: BitBoard,
    uncaptured_candidates: [BitBoard; NUM_UNCAPTURES],
    uncaptured_index: usize,
}

/// A simple routine to initialize the "uncaptured candidates" of a given board.
/// That is, a `BitBoard` for every element in [UNCAPTURES] specifying the
/// squares where the relevant piece type (or `None`) may have been uncaptured.
///
/// This simple routine just attends to material information (e.g. if all 8
/// white pawns and 2 white knights are on the board, we should not uncapture
/// white knights on any square). This information can be further refined
/// through a Sherlock analysis.
fn uncaptured_candidates(board: &RetractableBoard) -> [BitBoard; NUM_UNCAPTURES] {
    let color_pieces = board.color_combined(board.side_to_move());
    let pawns = board.pieces(Piece::Pawn) & color_pieces;
    let knights = board.pieces(Piece::Knight) & color_pieces;
    let bishops = board.pieces(Piece::Bishop) & color_pieces;
    let rooks = board.pieces(Piece::Rook) & color_pieces;
    let queens = board.pieces(Piece::Queen) & color_pieces;

    let lower_bound_promoted = max(0, knights.popcnt() as i32 - 2)
        + max(0, (bishops & LIGHT_SQUARES).popcnt() as i32 - 1)
        + max(0, (bishops & DARK_SQUARES).popcnt() as i32 - 1)
        + max(0, rooks.popcnt() as i32 - 2)
        + max(0, queens.popcnt() as i32 - 1);

    let lower_bound_nb_pawn_souls = pawns.popcnt() + lower_bound_promoted as u32;

    if lower_bound_nb_pawn_souls > 8 {
        return [EMPTY; NUM_UNCAPTURES];
    }

    let mut uncaptured_pawns = !PROMOTION_RANKS;
    let mut uncaptured_knights = !EMPTY;
    let mut uncaptured_bishops = !EMPTY;
    let mut uncaptured_rooks = !EMPTY;
    let mut uncaptured_queens = !EMPTY;

    if lower_bound_nb_pawn_souls == 8 {
        uncaptured_pawns = EMPTY;
        if knights.popcnt() >= 2 {
            uncaptured_knights = EMPTY;
        }
        if (bishops & DARK_SQUARES).popcnt() >= 1 {
            uncaptured_bishops &= LIGHT_SQUARES;
        }
        if (bishops & LIGHT_SQUARES).popcnt() >= 1 {
            uncaptured_bishops &= DARK_SQUARES;
        }
        if rooks.popcnt() >= 2 {
            uncaptured_rooks = EMPTY;
        }
        if queens.popcnt() >= 1 {
            uncaptured_queens = EMPTY;
        }
    }
    [
        !EMPTY,
        uncaptured_pawns,
        uncaptured_knights,
        uncaptured_bishops,
        uncaptured_rooks,
        uncaptured_queens,
    ]
}

impl RetractionGen {
    /// Create a new `RetractionGen` structure, only generating legal
    /// retractions, i.e. retractions that do not leave the king of the
    /// non-retracting player in check.
    #[inline(always)]
    pub fn new_legal(board: &RetractableBoard) -> Self {
        RetractionGen {
            retractions: RetractionGen::enumerate_retractions(board),
            index: 0,
            targets_mask: !EMPTY,
            uncaptured_candidates: uncaptured_candidates(board),
            uncaptured_index: 0,
        }
    }

    /// Refines the iterator on moves with the information provided from the
    /// board `Analysis`.
    /// TODO: Can we do better? ATM this routine is very simple.
    #[inline(always)]
    pub fn refine_iterator(&mut self, analysis: &Analysis) {
        // Only the pieces of the side to move matter.
        let color = analysis.board.side_to_move();
        for (i, uncaptured_piece) in UNCAPTURES.iter().enumerate() {
            if let Some(piece) = uncaptured_piece {
                let mut piece_uncaptured = EMPTY;
                for square in ALL_SQUARES {
                    if analysis.missing(color).all() & origins_of_piece_on(*piece, square) != EMPTY
                    {
                        piece_uncaptured ^= BitBoard::from_square(square);
                    }
                }
                self.uncaptured_candidates[i] &= piece_uncaptured;
            }
        }

        let mut captures = EMPTY;
        let mut nb_captures = 0;
        for origin in COLOR_ORIGINS[(!color).to_index()] {
            let tombs = analysis.captures(origin);
            nb_captures += tombs.popcnt();
            captures |= tombs;
        }

        if nb_captures == analysis.missing(color).size() {
            for i in 1..NUM_UNCAPTURES {
                self.uncaptured_candidates[i] &= captures;
            }
        }
    }

    #[inline(always)]
    fn enumerate_retractions(board: &RetractableBoard) -> RetractionList {
        let checkers = *board.checkers();
        let mask = !board.color_combined(board.side_to_move());
        let mut retraction_list = NoDrop::new(ArrayVec::<SourceAndTargets, BUFFER_SIZE>::new());

        if let EnPassantFlag::Some(src) = board.en_passant() {
            unsafe {
                retraction_list.push_unchecked(SourceAndTargets::new(
                    src,
                    get_file(src.get_file()) & get_rank((!board.side_to_move()).to_second_rank()),
                    UnCaptureKind::Forbidden,
                    false,
                ));
            }
            return retraction_list;
        }

        if checkers == EMPTY {
            PawnType::legals::<NotInCheck>(&mut retraction_list, board, mask);
            KnightType::legals::<NotInCheck>(&mut retraction_list, board, mask);
            BishopType::legals::<NotInCheck>(&mut retraction_list, board, mask);
            RookType::legals::<NotInCheck>(&mut retraction_list, board, mask);
            QueenType::legals::<NotInCheck>(&mut retraction_list, board, mask);
            KingType::legals::<NotInCheck>(&mut retraction_list, board, mask);
        } else if checkers.popcnt() == 1 {
            PawnType::legals::<InSimpleCheck>(&mut retraction_list, board, mask);
            KnightType::legals::<InSimpleCheck>(&mut retraction_list, board, mask);
            BishopType::legals::<InSimpleCheck>(&mut retraction_list, board, mask);
            RookType::legals::<InSimpleCheck>(&mut retraction_list, board, mask);
            QueenType::legals::<InSimpleCheck>(&mut retraction_list, board, mask);
            KingType::legals::<InSimpleCheck>(&mut retraction_list, board, mask);
        } else if checkers.popcnt() == 2 {
            PawnType::legals::<InDoubleCheck>(&mut retraction_list, board, mask);
            KnightType::legals::<InDoubleCheck>(&mut retraction_list, board, mask);
            BishopType::legals::<InDoubleCheck>(&mut retraction_list, board, mask);
            RookType::legals::<InDoubleCheck>(&mut retraction_list, board, mask);
            // King moves cannot deliver double checks and I bet we can forget
            // about queens too. Double check this (literally)!
        }

        retraction_list
    }
}

impl Iterator for RetractionGen {
    type Item = ChessRetraction;

    /// Find the next chess retraction.
    fn next(&mut self) -> Option<ChessRetraction> {
        if self.index >= self.retractions.len() {
            return None;
        }

        if self.retractions[self.index].targets & self.targets_mask == EMPTY {
            self.index += 1;
            return self.next();
        }

        let retraction = &mut self.retractions[self.index];
        let target = (retraction.targets & self.targets_mask).to_square();

        if retraction.uncapture_kind == UnCaptureKind::UnEnPassant {
            retraction.targets ^= BitBoard::from_square(target);
            return Some(ChessRetraction::new(retraction.source, target, None, false));
        };

        if self.uncaptured_index >= NUM_UNCAPTURES {
            retraction.targets ^= BitBoard::from_square(target);
            self.uncaptured_index = 0;
            return self.next();
        }

        let uncaptured = UNCAPTURES[self.uncaptured_index];
        let uncaptured_mask = self.uncaptured_candidates[self.uncaptured_index];

        if uncaptured_mask & BitBoard::from_square(retraction.source) == EMPTY
            || retraction.uncapture_kind == UnCaptureKind::Necessary && uncaptured.is_none()
            || retraction.uncapture_kind == UnCaptureKind::Forbidden && uncaptured.is_some()
        {
            self.uncaptured_index += 1;
            return self.next();
        }

        self.uncaptured_index += 1;
        Some(ChessRetraction::new(
            retraction.source,
            target,
            uncaptured,
            retraction.unpromotion,
        ))
    }
}

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use chess::Board;

#[test]
fn test_nb_retractions() {
    [
        ("8/4n3/4P2p/3k3R/7P/7K/8/8 b - -", 6),
        ("8/8/4P2p/3k3R/7P/7K/8/8 b - -", 7),
        ("8/8/3kP3/8/3R1Q2/8/4K3/8 b - -", 1),
        ("4k3/8/P7/8/8/8/8/4K2R b K -", 7),
        ("K7/RP3k2/n7/8/8/8/8/8 b - -", 10),
        ("8/8/8/8/8/4k3/8/r3K3 w - -", 40),
        ("r3K3/8/4k3/8/8/8/8/8 w - -", 35),
        ("6N1/8/7k/8/8/8/8/7K b - -", 19),
        ("6B1/5R1k/8/8/8/8/8/7K b - -", 1),
        ("8/8/8/8/4P3/7p/k6R/7K b - -", 6),
        ("8/8/8/8/4P3/2kp1p2/8/4K2R b K -", 2),
        ("8/8/8/8/4P3/3k1p2/8/4K2R b K -", 1),
        ("8/8/8/8/8/5k1N/8/6Kq w - -", 4),
        ("8/8/4k3/5P2/2B5/8/8/6K1 b - -", 0),
        ("1k6/3P4/8/8/8/8/7B/6K1 b - -", 1),
        ("3kQ3/8/8/8/8/8/4K3/3R4 b - -", 4),
        ("8/8/3k4/4P3/8/8/3K4/3R4 b - -", 11),
        ("8/8/3k4/4P3/8/8/4K3/3R4 b - -", 5),
        ("1k5N/3K3r/7N/4p3/8/8/8/8 w - -", 1),
        ("1k6/6b1/8/8/8/2p5/1K6/8 w - -", 11),
        ("N6K/2p5/1k6/8/8/8/8/8 b - -", 5),
        ("N6K/2pk4/8/8/8/8/8/8 b - -", 20),
        ("N7/2pk4/8/8/8/8/8/4K2R b K -", 5),
        ("8/8/8/1P3r2/BpPk4/1p1b4/P5PP/R3K3 b Q -", 1),
        ("4k2r/8/8/8/8/3P1P2/4p3/4K3 w k -", 1),
        ("8/8/8/8/6P1/5N1p/5K1P/4N1Bk w - -", 1),
        ("8/4k3/8/KP4Pp/pP6/8/8/8 w - h6", 1),
        ("k7/8/2K5/8/8/8/8/8 w - -", 10),
        ("2kr3K/3p4/8/8/8/8/q7/8 w - -", 1),
        ("2kr3K/3p4/8/8/8/8/8/8 w - -", 1),
        ("2kr3K/3p4/8/8/8/8/b7/8 w - -", 7),
        ("2kr1N2/1p1p4/8/N7/K7/8/8/8 w - -", 16),
        ("2kr1N2/1p1p4/8/8/8/6B1/8/2K5 w - -", 16),
        ("2kr1N2/1p1p4/4N3/N7/K7/8/8/8 w - -", 15),
        ("2kr1N2/1p1p4/5N2/N7/K7/8/8/8 w - -", 15),
        ("2kr1N2/1p1p4/8/N6B/K7/8/8/8 w - -", 15),
        ("2kr1N2/1p1p4/6P1/N6B/K7/8/8/8 w - -", 16),
        ("2kr1N2/K2p4/8/8/8/8/8/8 w - -", 10),
        ("1Nkr1N2/1p1p4/8/8/K7/8/8/8 w - -", 10),
        ("2kr1N2/K2p4/8/8/8/8/8/8 w - -", 10),
        ("2kr1n2/8/8/3K4/8/8/8/8 w - -", 16),
        ("6k1/8/8/8/8/8/5PP1/3n1RK1 b - -", 16),
        ("7k/8/8/8/7n/8/5PP1/3n1RK1 b - -", 16),
        ("7k/8/8/8/8/8/5PP1/3n1RK1 b - -", 15),
        ("5k2/8/8/8/8/8/8/3Q1RK1 b - -", 11),
        ("2k5/8/8/4K3/8/7B/6P1/8 b - -", 12),
        ("2k5/8/8/8/8/8/2K5/1nRn4 b - -", 26),
        ("2k5/K3N3/7p/8/8/7B/6q1/8 b - -", 6),
        ("2k2N1R/K7/7p/8/8/7B/6q1/8 b - -", 8),
        ("2k2B1R/K7/7p/8/8/8/8/1nRn4 b - -", 4),
        ("2k2N1R/8/7p/8/8/8/8/R3K3 b Q -", 39),
        ("2k4R/K3N3/8/8/8/8/8/8 b - -", 6),
        ("2k2R2/K7/5p2/1B5B/8/8/8/8 b - -", 34),
        ("2k2R2/K4p2/8/1B5B/8/8/8/8 b - -", 22),
        ("2k2R2/K7/8/5B2/8/8/8/8 b - -", 0),
        ("2k1R3/K7/8/5B2/8/8/8/8 b - -", 5),
        ("2k4R/K7/4B3/8/8/8/8/8 b - -", 6),
        ("BQRNNRQB/8/1PPPPPPP/8/8/8/8/2k3K1 b - -", 244),
    ]
    .iter()
    .for_each(|(fen, n)| {
        let board = Board::from_str(fen).unwrap();
        let mut retractable_board: RetractableBoard = board.into();
        retractable_board.set_uncertain_ep();
        let iterable = RetractionGen::new_legal(&retractable_board);
        let mut cnt = 0;
        for _r in iterable {
            cnt += 1;
        }
        println!("{}", fen);
        assert_eq!(cnt, *n);
    })
}
