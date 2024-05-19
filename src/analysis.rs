use std::fmt;

use chess::{
    get_bishop_rays, get_rank, get_rook_rays, BitBoard, Board, Color, File, Piece, Square,
    ALL_COLORS, ALL_FILES, ALL_PIECES, ALL_SQUARES, EMPTY, NUM_COLORS, NUM_FILES, NUM_PIECES,
    NUM_PROMOTION_PIECES, NUM_SQUARES, PROMOTION_PIECES,
};

use crate::{
    rules::ALL_ORIGINS,
    utils::{prom_index, MobilityGraph, UncertainSet},
};

pub(crate) struct Counter<T> {
    pub(crate) value: T,
    counter: usize,
}

impl<T> Counter<T> {
    fn new(value: T) -> Self {
        Self { value, counter: 1 }
    }

    pub(crate) fn counter(&self) -> usize {
        self.counter
    }
}

/// The result of a legality analysis.
#[derive(PartialOrd, PartialEq, Eq, Copy, Clone, Debug)]
pub enum Legality {
    /// A position is legal if it is reachable from the starting position via a
    /// sequence of legal moves.
    Legal,
    /// A position is illegal if it is unreachable from the starting position,
    /// i.e., it can never occur in an actual game.
    Illegal,
}

/// Errors that may result from the interaction with our API.
#[derive(PartialOrd, PartialEq, Eq, Copy, Clone, Debug)]
pub enum Error {
    /// The given square was expected to belong in the 1st, 2nd, 7th or 8th
    /// ranks.
    NotOriginSquare,
}

/// This type contains all the information that has been derived about the
/// legality of the position of interest.
pub struct Analysis {
    /// The position being analyzed.
    pub(crate) board: Board,

    /// A set of squares of steady pieces (that have certainly never moved and
    /// are still on their starting square).
    pub(crate) steady: Counter<BitBoard>,

    /// The candidate origins of the pieces that are still on the board.
    ///
    /// For `s : Square`, `origins[s.to_index()]` is a `BitBoard` encoding the
    /// squares where the piece currently on `s` may have started the game.
    ///
    /// If `BitBoard::from_square(t) & origins[s.to_index()] == EMPTY`, then
    /// the piece on `s` has definitely not started the game on square `t`.
    pub(crate) origins: Counter<[BitBoard; NUM_SQUARES]>,

    /// The candidate locations where pieces may have ended the game, i.e.,
    /// where they were captured or where they are currently standing.
    ///
    /// For `s : Square`, `destinies[s.to_index()]` is a `BitBoard` encoding the
    /// squares where the piece that started on `s` may have ended the game.
    ///
    /// If `BitBoard::from_square(t) & destinies[s.to_index()] == EMPTY`, then
    /// the piece which started on `s` has definitely not ended the game on `t`.
    pub(crate) destinies: Counter<[BitBoard; NUM_SQUARES]>,

    /// The candidate squares that may have been reached by a certain piece.
    ///
    /// For `s : Square`, `reachable[s.to_index()]` is a `BitBoard` encoding
    /// the squares where the piece that started on `s` may have reached
    /// during the game.
    ///
    /// If `BitBoard::from_square(t) & reachable[s.to_index()] == EMPTY`, then
    /// the piece which started on `s` has definitely not reached square `t`.
    pub(crate) reachable: Counter<[BitBoard; NUM_SQUARES]>,

    /// The squares that may have been reached by officers from their origin.
    ///
    /// `reachable_from_origin[c.to_index()][f.to_index()]`, for `c : Color` and
    /// `f : File`, is a `BitBoard` encoding the squares that may have been
    /// reached by the officer of color `c` that started the game on file `f`
    /// (and their relative 1st rank). The officer type is implied by the file.
    ///
    /// The 0's in the corresponding `BitBoard` represent squares that are
    /// definitely not reachable. The 1's are squares that have not yet been
    /// proven to be unreachable.
    pub(crate) reachable_from_origin: Counter<[[BitBoard; NUM_FILES]; NUM_COLORS]>,

    /// The squares that may have been reached by a promoted piece.
    ///
    /// `reachable_from_promotion[c.to_index()][prom_index(p)][f.to_index()]`,
    /// for `c : Color`, `p : Piece`, `f : File`,  is a `BitBoard`
    /// encoding the squares that may have been reached by a piece of color
    /// `c` that has promoted on file `f` (and their relative 8th rank) into
    /// piece type `p`.
    ///
    /// The 0's in the corresponding `BitBoard` represent squares that are
    /// definitely not reachable. The 1's are squares that have not yet been
    /// proven to be unreachable.
    pub(crate) reachable_from_promotion:
        Counter<[[[BitBoard; NUM_FILES]; NUM_PROMOTION_PIECES]; NUM_COLORS]>,

    /// The minimum number of captures necessary for a pawn to reach targets.
    ///
    /// `pawn_capture_distances[c.to_index()][f.to_index()][s.to_index()]`, for
    /// `c : Color`, `f : File`, `s : Square`, is a lower bound on the number of
    /// captures necessary for the pawn of color `c` that started on file `f` to
    /// reach square `s` as a pawn.
    ///
    /// Unreachable squares store a value of 16 by default.
    pub(crate) pawn_capture_distances: Counter<[[[u8; NUM_SQUARES]; NUM_FILES]; NUM_COLORS]>,

    /// The squares where a pawn must have captured in order to reach a target.
    ///
    /// `pawn_forced_captures[c.to_index()][f.to_index()][s.to_index()]`, for
    /// `c : Color`, `f : File`, `s : Square`, is a `BitBoard` encoding the
    /// squares where the pawn of color `c` that started on file `f` must
    /// have captured to reach square `s` as a pawn.
    pub(crate) pawn_forced_captures: Counter<[[[BitBoard; NUM_SQUARES]; NUM_FILES]; NUM_COLORS]>,

    /// The squares where the missing pieces of each color started the game.
    ///
    /// For `c : Color`, `missing[c.to_index()]` is an `UncertainSet` encoding
    /// the squares where the missing pieces of color `c` started the game.
    pub(crate) missing: Counter<[UncertainSet; NUM_COLORS]>,

    /// The squares where opponent pieces have certainly been captured.
    ///
    /// For `s : Square`, `tombs[s.to_index()]` is a `BitBoard` encoding
    /// a set of squares where the piece that started on `s` has certainly
    /// captured an enemy piece.
    pub(crate) tombs: Counter<[BitBoard; NUM_SQUARES]>,

    /// A lower-upper bound pair on the number of captures performed by every
    /// piece.
    ///
    /// For `s : Square`, `captures_bounds[s.to_index()] = (l, u)` means that
    /// the number of captures, `n` performed by the piece that started the
    /// game on `s` is such that `l <= n <= u`.
    pub(crate) captures_bounds: Counter<[(i32, i32); NUM_SQUARES]>,

    /// Mobility graphs, for each color and piece type, where nodes are squares
    /// and arrows indicate the possible moves that a piece of the
    /// corresponding type and color can have performed during a game leading to
    /// the position of interest.
    pub(crate) mobility: Counter<[[MobilityGraph; NUM_PIECES]; NUM_COLORS]>,

    /// A flag about the legality of the position. `None` if undetermined,
    /// `Some(true)` if the position has been determined to be illegal, and
    /// `Some(false)` if the position is known to be legal.
    pub(crate) result: Option<Legality>,
}

impl Analysis {
    /// Initializes a legality analysis for the given board.
    pub fn new(board: &Board) -> Self {
        Analysis {
            board: *board,
            steady: Counter::new(EMPTY),
            origins: Counter::new([!EMPTY; NUM_SQUARES]),
            destinies: Counter::new([!EMPTY; NUM_SQUARES]),
            reachable: Counter::new([!EMPTY; NUM_SQUARES]),
            reachable_from_origin: Counter::new([[!EMPTY; NUM_FILES]; NUM_COLORS]),
            reachable_from_promotion: Counter::new(
                [[[!EMPTY; NUM_FILES]; NUM_PROMOTION_PIECES]; NUM_COLORS],
            ),
            pawn_capture_distances: Counter::new([[[0; NUM_SQUARES]; NUM_FILES]; NUM_COLORS]),
            pawn_forced_captures: Counter::new([[[EMPTY; NUM_SQUARES]; NUM_FILES]; NUM_COLORS]),
            missing: Counter::new([
                UncertainSet::new(16 - board.color_combined(Color::White).popcnt()),
                UncertainSet::new(16 - board.color_combined(Color::Black).popcnt()),
            ]),
            tombs: Counter::new([EMPTY; NUM_SQUARES]),
            captures_bounds: Counter::new([(0, 15); NUM_SQUARES]),
            mobility: Counter::new([
                core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::White)),
                core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::Black)),
            ]),
            result: None,
        }
    }

    /// The squares that may have been reached by the piece that started on the
    /// given square.
    #[inline]
    pub(crate) fn reachable(&self, square: Square) -> BitBoard {
        self.reachable.value[square.to_index()]
    }

    /// The squares that may have been reached by the officer of the given color
    /// that started the game on the given file.
    pub(crate) fn reachable_from_origin(&self, color: Color, file: File) -> BitBoard {
        self.reachable_from_origin.value[color.to_index()][file.to_index()]
    }

    /// The squares that may have been reached by the piece of the given type
    /// and color that has just promoted on the given file.
    pub(crate) fn reachable_from_promotion(
        &self,
        color: Color,
        piece: Piece,
        file: File,
    ) -> BitBoard {
        self.reachable_from_promotion.value[color.to_index()][prom_index(piece)][file.to_index()]
    }

    /// The minimum number of captures necessary for the pawn of the given color
    /// and the given file to reach the given target as a pawn, from its origin
    /// square.
    pub(crate) fn pawn_capture_distances(&self, color: Color, file: File, target: Square) -> u8 {
        self.pawn_capture_distances.value[color.to_index()][file.to_index()][target.to_index()]
    }

    /// The squares where the pawn of the given color that started on the given
    /// file must have captured in order to reach the given target.
    pub(crate) fn pawn_forced_captures(
        &self,
        color: Color,
        file: File,
        target: Square,
    ) -> BitBoard {
        self.pawn_forced_captures.value[color.to_index()][file.to_index()][target.to_index()]
    }

    /// The missing pieces of the given color.
    pub(crate) fn missing(&self, color: Color) -> UncertainSet {
        self.missing.value[color.to_index()]
    }

    /// The squares where the piece that started on the given square has
    /// certainly captured opponents pieces.
    pub(crate) fn tombs(&self, square: Square) -> BitBoard {
        self.tombs.value[square.to_index()]
    }

    /// The known lower bound on the number of captures performed by the piece
    /// that started the game on the given square.

    pub(crate) fn nb_captures_lower_bound(&self, square: Square) -> i32 {
        self.captures_bounds.value[square.to_index()].0
    }

    /// The known upper bound on the number of captures performed by the piece
    /// that started the game on the given square.
    #[inline]
    pub(crate) fn nb_captures_upper_bound(&self, square: Square) -> i32 {
        self.captures_bounds.value[square.to_index()].1
    }

    /// The piece type of the piece on the given square in the analysis's board.
    /// Panics if the square is empty.
    pub(crate) fn piece_type_on(&self, square: Square) -> Piece {
        self.board.piece_on(square).unwrap()
    }

    /// The piece color of the piece on the given square in the analysis's
    /// board. Panics if the square is empty.
    pub(crate) fn piece_color_on(&self, square: Square) -> Color {
        for color in ALL_COLORS {
            if BitBoard::from_square(square) & self.board.color_combined(color) != EMPTY {
                return color;
            }
        }
        panic!("piece_color_on: the given square should not be empty");
    }
}

impl Analysis {
    /// Update the information on steady pieces with the given value.
    pub(crate) fn update_steady(&mut self, value: BitBoard) -> bool {
        if (self.steady.value | value) == self.steady.value {
            return false;
        }
        self.steady.value |= value;
        self.steady.counter += 1;
        true
    }

    /// Update the candidate origins of the piece on the given square, with the
    /// given value.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn update_origins(&mut self, square: Square, value: BitBoard) -> bool {
        let new_origins = self.origins.value[square.to_index()] & value;
        if self.origins.value[square.to_index()] == new_origins {
            return false;
        }
        self.origins.value[square.to_index()] = new_origins;
        self.origins.counter += 1;

        // if the set of candidate origins of a piece is empty, the position is illegal
        if value == EMPTY {
            self.result = Some(Legality::Illegal);
        }
        true
    }

    /// Update the candidate destinies of the piece that started on the given
    /// square, with the given value.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn update_destinies(&mut self, square: Square, value: BitBoard) -> bool {
        let new_destinies = self.destinies.value[square.to_index()] & value;
        if self.destinies.value[square.to_index()] == new_destinies {
            return false;
        }
        self.destinies.value[square.to_index()] = new_destinies;
        self.destinies.counter += 1;

        // if the set of candidate destinies of a piece is empty, the position is
        // illegal
        if value == EMPTY {
            self.result = Some(Legality::Illegal);
        }
        true
    }

    /// Update the reachable squares of the piece that started on the given
    /// square, with the given value.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn update_reachable(&mut self, square: Square, value: BitBoard) -> bool {
        let new_reachable = self.reachable.value[square.to_index()] & value;
        if self.reachable.value[square.to_index()] == new_reachable {
            return false;
        }
        self.reachable.value[square.to_index()] = new_reachable;
        self.reachable.counter += 1;
        true
    }

    /// Update the reachable squares of the officer of the given color that
    /// started on the given file, with the given value. Returns a boolean
    /// value indicating whether the update changed anything.
    pub(crate) fn update_reachable_from_origin(
        &mut self,
        color: Color,
        file: File,
        value: BitBoard,
    ) -> bool {
        let reachable = self.reachable_from_origin(color, file);
        let new_reachable = reachable & value;
        if reachable == new_reachable {
            return false;
        }
        self.reachable_from_origin.value[color.to_index()][file.to_index()] = new_reachable;
        self.reachable_from_origin.counter += 1;
        true
    }

    /// Update the reachable squares of a promoted piece of the given color and
    /// piece type that promoted on the given file, with the given value.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn update_reachable_from_promotion(
        &mut self,
        color: Color,
        piece: Piece,
        file: File,
        value: BitBoard,
    ) -> bool {
        let reachable = self.reachable_from_promotion(color, piece, file);
        let new_reachable = reachable & value;
        if reachable == new_reachable {
            return false;
        }
        self.reachable_from_promotion.value[color.to_index()][prom_index(piece)][file.to_index()] =
            new_reachable;
        self.reachable_from_promotion.counter += 1;
        true
    }

    /// Update the information on pawn capture distances for the pawn of the
    /// given color that started on the given file, with the given distances.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn update_pawn_capture_distances(
        &mut self,
        color: Color,
        file: File,
        distances: &[u8; NUM_SQUARES],
    ) -> bool {
        let mut progress = false;
        for target in ALL_SQUARES {
            let distance = distances[target.to_index()];
            if self.pawn_capture_distances(color, file, target) < distance {
                progress = true;
                self.pawn_capture_distances.value[color.to_index()][file.to_index()]
                    [target.to_index()] = distance;
            }
        }
        if progress {
            self.pawn_capture_distances.counter += 1;
        }
        progress
    }

    /// Update the information on pawn forced captures for the pawn of the
    /// given color that started on the given file, for going to the given
    /// target, with the given value.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn update_pawn_forced_captures(
        &mut self,
        color: Color,
        file: File,
        target: Square,
        value: BitBoard,
    ) -> bool {
        let forced = self.pawn_forced_captures(color, file, target);
        let new_forced = forced | value;
        if forced == new_forced {
            return false;
        }
        self.pawn_forced_captures.value[color.to_index()][file.to_index()][target.to_index()] =
            new_forced;
        self.pawn_forced_captures.counter += 1;
        true
    }

    /// Update the information of missing pieces of the given color, with a
    /// given set of pieces that are certainly not missing.
    pub(crate) fn update_certainly_not_missing(&mut self, color: Color, value: BitBoard) -> bool {
        self.missing.value[color.to_index()].remove(value)
    }

    /// Update the information of missing pieces of the given color, with a
    /// given set of pieces that are certainly missing.
    pub(crate) fn update_certainly_missing(&mut self, color: Color, value: BitBoard) -> bool {
        self.missing.value[color.to_index()].add(value)
    }

    /// Update the tombs of the piece that started on the given square, with the
    /// given value.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn update_tombs(&mut self, square: Square, value: BitBoard) -> bool {
        let new_tombs = self.tombs.value[square.to_index()] | value;
        if self.tombs.value[square.to_index()] == new_tombs {
            return false;
        }
        self.tombs.value[square.to_index()] = new_tombs;
        self.tombs.counter += 1;
        true
    }

    /// Updates the mobility graph of the given piece and the given color, by
    /// removing all connections from the given square.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn remove_outgoing_edges(
        &mut self,
        piece: Piece,
        color: Color,
        square: Square,
    ) -> bool {
        let progress =
            self.mobility.value[color.to_index()][piece.to_index()].remove_outgoing_edges(square);
        if progress {
            self.mobility.counter += 1
        }
        progress
    }

    /// Updates the mobility graph of the given piece and the given color, by
    /// removing all connections into the given square.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn remove_incoming_edges(
        &mut self,
        piece: Piece,
        color: Color,
        square: Square,
    ) -> bool {
        let progress =
            self.mobility.value[color.to_index()][piece.to_index()].remove_incoming_edges(square);
        if progress {
            self.mobility.counter += 1
        }
        progress
    }

    /// Updates the mobility graph of the given piece and the given color, by
    /// removing all the connections that pass through the given square.
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn remove_edges_passing_through_square(
        &mut self,
        piece: Piece,
        color: Color,
        square: Square,
    ) -> bool {
        let mut progress = false;
        for source in get_rook_rays(square) | get_bishop_rays(square) {
            for target in chess::line(square, source)
                & !BitBoard::from_square(square)
                & !BitBoard::from_square(source)
            {
                if (BitBoard::from_square(square) & chess::between(source, target)) != EMPTY {
                    progress |= self.mobility.value[color.to_index()][piece.to_index()]
                        .remove_edge(source, target);
                }
            }
        }
        if progress {
            self.mobility.counter += 1
        }
        progress
    }

    /// Updates the mobility graph of the given piece and the given color, by
    /// removing all the connections that pass in a line through the two given
    /// squares (inclusive, i.e. moves that go from square1 to square2 are also
    /// removed).
    /// Returns a boolean value indicating whether the update changed anything.
    pub(crate) fn remove_edges_passing_through_squares(
        &mut self,
        piece: Piece,
        color: Color,
        square1: Square,
        square2: Square,
    ) -> bool {
        debug_assert_ne!(square1, square2);
        let mut progress = false;
        let squares = BitBoard::from_square(square1) | BitBoard::from_square(square2);
        for source in chess::line(square1, square2) {
            for target in chess::line(square1, square2) & !BitBoard::from_square(source) {
                // the squares between source and target, including these
                let segment = chess::between(source, target)
                    | BitBoard::from_square(source)
                    | BitBoard::from_square(target);
                // if both square1 and square2 are included in the segment
                if squares & segment == squares {
                    progress |= self.mobility.value[color.to_index()][piece.to_index()]
                        .remove_edge(source, target);
                }
            }
        }
        if progress {
            self.mobility.counter += 1
        }
        progress
    }

    /// Update the known lower bound on the number of captures performed by the
    /// piece that started the game on the given square, with the given
    /// value.
    pub(crate) fn update_captures_lower_bound(&mut self, square: Square, bound: i32) -> bool {
        if self.captures_bounds.value[square.to_index()].0 >= bound {
            return false;
        }
        self.captures_bounds.value[square.to_index()].0 = bound;
        self.captures_bounds.counter += 1;
        true
    }

    /// Update the known upper bound on the number of captures performed by the
    /// piece that started the game on the given square, with the given
    /// value.
    pub(crate) fn update_captures_upper_bound(&mut self, square: Square, bound: i32) -> bool {
        if self.captures_bounds.value[square.to_index()].1 <= bound {
            return false;
        }
        self.captures_bounds.value[square.to_index()].1 = bound;
        self.captures_bounds.counter += 1;
        true
    }
}

fn write_bitboard(f: &mut fmt::Formatter, name: String, bitboard: BitBoard) -> fmt::Result {
    if bitboard == !EMPTY {
        writeln!(f, "  {}: ALL", name)?;
    } else {
        let (negated, bb) = if bitboard.popcnt() < 40 {
            (" ", bitboard)
        } else {
            ("!", !bitboard)
        };
        write!(f, "  {}: {}{{ ", name, negated)?;
        for element in bb {
            write!(f, "{} ", element)?;
        }
        writeln!(f, "}}")?;
    }
    Ok(())
}

impl fmt::Display for Analysis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FEN: {}", self.board,)?;
        writeln!(f, "\nsteady (cnt: {}):\n", self.origins.counter())?;
        write_bitboard(f, String::from("steady"), self.steady.value)?;
        writeln!(f, "\norigins (cnt: {}):\n", self.origins.counter())?;
        for square in *self.board.combined() {
            write_bitboard(f, square.to_string(), self.origins.value[square.to_index()])?;
        }
        writeln!(f, "\ndestinies (cnt: {}):\n", self.destinies.counter())?;
        for square in ALL_ORIGINS {
            write_bitboard(
                f,
                square.to_string(),
                self.destinies.value[square.to_index()],
            )?;
        }
        writeln!(f, "\nreachable (cnt: {}):\n", self.reachable.counter())?;
        for square in ALL_ORIGINS {
            write_bitboard(f, square.to_string(), self.reachable(square))?;
        }
        writeln!(
            f,
            "\nreachable_from_origin (cnt: {}):",
            self.reachable_from_origin.counter()
        )?;
        for color in ALL_COLORS {
            writeln!(f, "\n {:?}:", color)?;
            for file in ALL_FILES {
                let rank = color.to_my_backrank();
                let square = Square::make_square(rank, file);
                let reachable = self.reachable_from_origin(color, file);
                write_bitboard(f, square.to_string(), reachable)?;
            }
        }
        writeln!(
            f,
            "\nreachable_from_promotion (cnt: {}):",
            self.reachable_from_promotion.counter()
        )?;
        for color in ALL_COLORS {
            for piece in PROMOTION_PIECES {
                writeln!(f, "\n {:?} {:?}:", color, piece)?;
                for file in ALL_FILES {
                    let rank = color.to_their_backrank();
                    let square = Square::make_square(rank, file);
                    let reachable = self.reachable_from_promotion(color, piece, file);
                    write_bitboard(f, square.to_string(), reachable)?;
                }
            }
        }
        writeln!(
            f,
            "\npawn_capture_distances (cnt: {}):",
            self.pawn_capture_distances.counter()
        )?;
        for color in ALL_COLORS {
            for file in ALL_FILES {
                let square = Square::make_square(color.to_second_rank(), file);
                if self.is_steady(square) {
                    continue;
                }
                write!(f, "\n  {:?} {:?}-pawn:", color, file)?;
                for d in 0..=6 {
                    write!(f, "\n    {}:", d)?;
                    for target in ALL_SQUARES {
                        if self.pawn_capture_distances(color, file, target) == d {
                            write!(f, " {}", target)?;
                        }
                    }
                }
                writeln!(f)?;
            }
        }
        writeln!(
            f,
            "\npawn_forced_captures (cnt: {}):",
            self.pawn_forced_captures.counter()
        )?;
        for color in ALL_COLORS {
            for file in ALL_FILES {
                for target in get_rank(color.to_their_backrank()) {
                    let forced = self.pawn_forced_captures(color, file, target);
                    if forced != EMPTY {
                        writeln!(
                            f,
                            "\n{:?}-{:?} -> {}:\n{}\n",
                            color,
                            file,
                            target,
                            forced.reverse_colors()
                        )?;
                    }
                }
            }
        }
        writeln!(f, "\nmissing (cnt: {}):\n", self.missing.counter())?;
        for color in ALL_COLORS {
            writeln!(f, "{:?} missing:\n{}", color, self.missing(color))?;
        }
        writeln!(f, "\ntombs (cnt: {}):\n", self.tombs.counter())?;
        for square in ALL_ORIGINS {
            write_bitboard(f, square.to_string(), self.tombs(square))?;
        }
        writeln!(
            f,
            "\ncaptures bounds (cnt: {}):\n",
            self.captures_bounds.counter
        )?;
        let mut lines = vec![];
        let mut line = vec![];
        let mut cnt = 0;
        for square in *Board::default().combined() {
            let lower = self.nb_captures_lower_bound(square);
            let upper = self.nb_captures_upper_bound(square);
            line.push(format!(" {}: ({}, {})", square, lower, upper));
            cnt += 1;
            if cnt % 8 == 0 {
                lines.push(line.join(" "));
                line = vec![];
                if cnt == 16 {
                    lines.push(String::new());
                }
            }
        }
        for line in lines.iter().rev() {
            writeln!(f, "{}", line)?;
        }
        writeln!(f, "\nresult: {:?}", self.result)
    }
}
