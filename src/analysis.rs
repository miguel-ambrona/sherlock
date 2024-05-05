use std::fmt;

use chess::{
    get_bishop_rays, get_rook_rays, BitBoard, Board, Color, Piece, Square, ALL_COLORS, ALL_PIECES,
    EMPTY, NUM_COLORS, NUM_PIECES, NUM_SQUARES,
};

use crate::{rules::ALL_ORIGINS, utils::MobilityGraph};

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

/// In the following examples, we will use the following reference position
/// designed to illustrate many different concepts.
///
/// ![Alt version](https://chasolver.org/FEN.png)
///
/// White to move and *no castling rights are enabled*.
impl Analysis {
    /// Initializes a legality analysis for the given board.
    pub fn new(board: &Board) -> Self {
        Analysis {
            board: *board,
            steady: Counter::new(EMPTY),
            origins: Counter::new([!EMPTY; NUM_SQUARES]),
            destinies: Counter::new([!EMPTY; NUM_SQUARES]),
            reachable: Counter::new([!EMPTY; NUM_SQUARES]),
            tombs: Counter::new([EMPTY; NUM_SQUARES]),
            captures_bounds: Counter::new([(0, 15); NUM_SQUARES]),
            mobility: Counter::new([
                core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::White)),
                core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::Black)),
            ]),
            result: None,
        }
    }

    /// Tells whether the piece on the given square was classified as steady
    /// (it has never moved and is still on their starting square).
    /// ```
    /// use std::str::FromStr;
    ///
    /// use chess::{Board, Square};
    /// use sherlock::analyze;
    ///
    /// let board = Board::from_str("rnbqk1Nr/ppp1pp1p/4p1p1/8/8/1P6/1PPPPPP1/1RBQKB1R w - -")
    ///     .expect("Valid Position");
    /// let analysis = analyze(&board);
    ///
    /// // The white queen cannot have possibly moved, unlike the black queen
    /// assert_eq!(analysis.is_steady(Square::D1), true);
    /// assert_eq!(analysis.is_steady(Square::D8), false);
    /// ```
    #[inline]
    pub fn is_steady(&self, square: Square) -> bool {
        BitBoard::from_square(square) & self.steady.value != EMPTY
    }

    /// The candidate origins of the piece that is on the given square in the
    /// analyzed board.
    /// ```
    /// use std::str::FromStr;
    ///
    /// use chess::{BitBoard, Board, Square, EMPTY};
    /// use sherlock::analyze;
    ///
    /// let board = Board::from_str("rnbqk1Nr/ppp1pp1p/4p1p1/8/8/1P6/1PPPPPP1/1RBQKB1R w - -")
    ///     .expect("Valid Position");
    /// let analysis = analyze(&board);
    ///
    /// // The piece on G8 (a white knight) may have started the game on B1 or G1,
    /// // but it can also be the H2-pawn promoted
    /// assert_eq!(
    ///     analysis.origins(Square::G8),
    ///     BitBoard::from_square(Square::B1)
    ///         | BitBoard::from_square(Square::G1)
    ///         | BitBoard::from_square(Square::H2)
    /// );
    ///
    /// // The pawn on E6 definitely comes from D7
    /// assert_eq!(
    ///     analysis.origins(Square::E6),
    ///     BitBoard::from_square(Square::D7)
    /// );
    ///
    /// // This should not be queried on an empty square, but if you insist...
    /// assert_eq!(analysis.origins(Square::E5), !EMPTY);
    /// ```
    #[inline]
    pub fn origins(&self, square: Square) -> BitBoard {
        self.origins.value[square.to_index()]
    }

    /// The candidate destinies of the piece that started on the given square.
    #[inline]
    pub fn destinies(&self, square: Square) -> BitBoard {
        self.destinies.value[square.to_index()]
    }

    /// The squares that may have been reached by the piece that started on the
    /// given square.
    #[inline]
    pub fn reachable(&self, square: Square) -> BitBoard {
        self.reachable.value[square.to_index()]
    }

    /// The known lower bound on the number of captures performed by the piece
    /// that started the game on the given square.
    #[inline]
    pub fn nb_captures_lower_bound(&self, square: Square) -> i32 {
        self.captures_bounds.value[square.to_index()].0
    }

    /// The known upper bound on the number of captures performed by the piece
    /// that started the game on the given square.
    #[inline]
    pub fn nb_captures_upper_bound(&self, square: Square) -> i32 {
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
    #[cfg(test)]
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

fn write_array(
    f: &mut fmt::Formatter,
    name: &str,
    array: &Counter<[BitBoard; NUM_SQUARES]>,
    squares: BitBoard,
) -> fmt::Result {
    writeln!(f, "\n{} (cnt: {}):\n", name, array.counter())?;
    for square in squares {
        if array.value[square.to_index()] == !EMPTY {
            writeln!(f, "  {}: ALL", square)?;
        } else {
            write!(f, "  {}: [", square)?;
            for element in array.value[square.to_index()] {
                write!(f, "{},", element)?;
            }
            writeln!(f, "]")?;
        }
    }
    Ok(())
}

impl fmt::Display for Analysis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FEN: {}\n", self.board,)?;
        writeln!(f, "steady:\n{}", self.steady.value.reverse_colors())?;
        write_array(f, "origins", &self.origins, *self.board.combined())?;
        write_array(f, "destinies", &self.destinies, ALL_ORIGINS)?;
        write_array(f, "reachable", &self.reachable, ALL_ORIGINS)?;
        write_array(f, "tombs", &self.tombs, ALL_ORIGINS)?;
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
