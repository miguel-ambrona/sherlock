use std::fmt;

use chess::{
    BitBoard, Board, Color, Piece, Square, ALL_COLORS, ALL_PIECES, EMPTY, NUM_COLORS, NUM_PIECES,
    NUM_SQUARES,
};

use crate::utils::MobilityGraph;

pub(crate) struct Counter<T> {
    pub(crate) value: T,
    counter: usize,
}

impl<T> Counter<T> {
    fn new(value: T) -> Self {
        Self { value, counter: 1 }
    }

    #[inline]
    pub(crate) fn counter(&self) -> usize {
        self.counter
    }

    #[inline]
    pub(crate) fn increase_counter(&mut self, progress: bool) {
        if progress {
            self.counter += 1
        }
    }
}

/// Type `Analysis` contains all the information that has been derived about the
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
    pub(crate) illegal: Option<bool>,

    /// A flag indicating whether there has been recent progress in updating the
    /// state (used to know when to stop applying rules).
    pub(crate) progress: bool,
}

impl Analysis {
    /// Initializes a proof state for the given board.
    pub fn new(board: &Board) -> Self {
        Analysis {
            board: *board,
            steady: Counter::new(EMPTY),
            origins: Counter::new([!EMPTY; 64]),
            destinies: Counter::new([!EMPTY; 64]),
            captures_bounds: Counter::new([(0, 15); 64]),
            mobility: Counter::new([
                core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::White)),
                core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::Black)),
            ]),
            illegal: None,
            progress: false,
        }
    }

    /// Tells whether the piece on the given square was classified as steady.
    #[inline]
    pub fn is_steady(&self, square: Square) -> bool {
        BitBoard::from_square(square) & self.steady.value != EMPTY
    }

    /// The candidate origins of the piece on the given square.
    #[inline]
    pub fn origins(&self, square: Square) -> BitBoard {
        self.origins.value[square.to_index()]
    }

    /// The candidate destinies of the piece that started on the given square.
    #[inline]
    pub fn destinies(&self, square: Square) -> BitBoard {
        self.destinies.value[square.to_index()]
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

    /// The piece type of the piece on the given square in the state's board.
    /// Panics if the square is empty.
    pub(crate) fn piece_type_on(&self, square: Square) -> Piece {
        self.board.piece_on(square).unwrap()
    }

    /// The piece color of the piece on the given square in the state's board.
    /// Panics if the square is empty.
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
        true
    }

    /// Update the candidate origins of the piece on the given square, with the
    /// given value.
    pub(crate) fn update_origins(&mut self, square: Square, value: BitBoard) -> bool {
        if self.origins.value[square.to_index()] == value {
            return false;
        }
        self.origins.value[square.to_index()] = value;
        // if the set of candidate origins of a piece is empty, the position is illegal
        if value == EMPTY {
            self.illegal = Some(true);
        }
        true
    }

    /// Update the candidate destinies of the piece that started on the given
    /// square, with the given value.
    /// Returns a boolean value indicating whether the update actually changed
    /// the known information on destinies.
    pub(crate) fn update_destinies(&mut self, square: Square, value: BitBoard) -> bool {
        if self.destinies.value[square.to_index()] == value {
            return false;
        }
        self.destinies.value[square.to_index()] = value;
        // if the set of candidate destinies of a piece is empty, the position is
        // illegal
        if value == EMPTY {
            self.illegal = Some(true);
        }
        true
    }

    /// Update the known lower bound on the number of captures performed by the
    /// piece that started the game on the given square, with the given
    /// value.
    #[cfg(test)]
    pub(crate) fn update_captures_lower_bound(&mut self, square: Square, bound: i32) {
        self.captures_bounds.value[square.to_index()].0 = bound;
    }

    /// Update the known upper bound on the number of captures performed by the
    /// piece that started the game on the given square, with the given
    /// value.
    pub(crate) fn update_captures_upper_bound(&mut self, square: Square, bound: i32) {
        self.captures_bounds.value[square.to_index()].1 = bound;
    }
}

impl fmt::Display for Analysis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FEN: {}\n", self.board,)?;
        writeln!(f, "steady:\n{}", self.steady.value.reverse_colors())?;
        writeln!(f, "\norigins (cnt: {}):\n", self.origins.counter)?;
        for square in *self.board.combined() & !self.steady.value {
            write!(f, "  {} <- [", square)?;
            for origin in self.origins(square) {
                write!(f, "{},", origin)?;
            }
            writeln!(f, "]")?;
        }
        writeln!(f, "\ndestinies (cnt: {}):\n", self.destinies.counter)?;
        for square in *Board::default().combined() & !self.steady.value {
            if self.destinies(square) == !EMPTY {
                writeln!(f, "  {}, -> ANY", square)?;
            } else {
                write!(f, "  {} -> [", square)?;
                for destiny in self.destinies(square) {
                    write!(f, "{},", destiny)?;
                }
                writeln!(f, "]")?;
            }
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
        writeln!(f, "\nillegal: {:?}", self.illegal)
    }
}
