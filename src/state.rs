use std::fmt;

use chess::{BitBoard, Board, Color, Piece, Square, ALL_COLORS, EMPTY};

pub type Counter = usize;

/// Type `State` contains all the information that has been derived about the
/// legality of the position of interest.

pub struct State {
    /// The position being analyzed.
    pub board: Board,

    /// A set of squares of steady pieces (that have certainly never moved and
    /// are still on their starting square).
    pub steady: BitBoard,

    /// The potential candidate origins of the pieces that are still on the
    /// board.
    ///
    /// For `s : Square`, `origins[s.to_index()]` is a `BitBoard` encoding the
    /// squares where the piece currently on `s` may have started the game.
    ///
    /// On the other hand, `BitBoard::from_square(t) & origins[s.to_index()] ==
    /// EMPTY` means that the piece on `s` has definitely not started the
    /// game on square `t`.
    ///
    /// We also store a counter that is increased every time this variable is
    /// updated.
    pub origins: ([BitBoard; 64], Counter),

    /// A flag about the legality of the position. `None` if undetermined,
    /// `Some(true)` if the position has been determined to be illegal, and
    /// `Some(false)` if the position is known to be legal.
    pub illegal: Option<bool>,

    /// A flag indicating whether there has been recent progress in updating the
    /// state (used to know when to stop applying rules).
    pub progress: bool,
}

impl State {
    /// Initializes a proof state for the given board.
    pub fn new(board: &Board) -> Self {
        State {
            board: *board,
            steady: EMPTY,
            origins: ([!EMPTY; 64], 0),
            illegal: None,
            progress: false,
        }
    }

    /// Tells whether or not the piece on the current state was classified as
    /// steady.
    #[inline]
    pub fn _is_steady(&self, square: Square) -> bool {
        BitBoard::from_square(square) & self.steady != EMPTY
    }

    /// The candidate origins of the piece on the given square.
    #[inline]
    pub fn origins(&self, square: Square) -> BitBoard {
        self.origins.0[square.to_index()]
    }

    /// The piece type of the piece on the given square in the state's board.
    /// Panics if the square is empty.
    #[inline]
    pub fn piece_type_on(&self, square: Square) -> Piece {
        self.board.piece_on(square).unwrap()
    }

    /// The piece color of the piece on the given square in the state's board.
    /// Panics if the square is empty.
    #[inline]
    pub fn piece_color_on(&self, square: Square) -> Color {
        for color in ALL_COLORS {
            if BitBoard::from_square(square) & self.board.color_combined(color) != EMPTY {
                return color;
            }
        }
        panic!("piece_color_on: the given square should not be empty");
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FEN: {}\n", self.board,)?;
        writeln!(f, "steady:\n{}", self.steady.reverse_colors())?;
        writeln!(f, "origins (cnt: {}):\n", self.origins.1)?;
        for square in *self.board.combined() & !self.steady {
            write!(f, "  {} <- [", square)?;
            for origin in self.origins(square) {
                write!(f, "{},", origin)?;
            }
            writeln!(f, "]")?;
        }
        writeln!(f, "illegal: {:?}", self.illegal)
    }
}
