use std::fmt;

use chess::{BitBoard, Board, EMPTY};

/// State containing all the information that has been derived about the
/// legality of the position of interest.
pub struct State {
    /// The position being analyzed.
    pub board: Board,
    /// A set of squares of steady pieces (that have certainly never moved and
    /// are still on their starting square).
    pub steady: BitBoard,
    /// A flag about the legality of the position. `None` if undetermined,
    /// `Some(true)` if the position has been determined to be illegal, and
    /// `Some(false)` if the position is known to be legal.
    pub illegal: Option<bool>,
    /// A flag indicating whether there has been recent progress in updating the
    /// state (used to know when to stop applying rules).
    pub progress: bool,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FEN: {}\n", self.board,)?;
        writeln!(f, "steady:\n{}", self.steady)?;
        writeln!(f, "illegal: {:?}", self.illegal)
    }
}

impl State {
    pub fn new(board: &Board) -> Self {
        State {
            board: board.clone(),
            steady: EMPTY,
            illegal: None,
            progress: false,
        }
    }
}
