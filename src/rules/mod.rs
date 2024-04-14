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

/// A legality rule, it updates the proof state about the legality of the
/// position, after deriving new information.
pub trait Rule: fmt::Debug {
    /// Initializes the rule state for a given board.
    fn new(board: &Board) -> Self
    where
        Self: Sized + fmt::Debug;

    /// Check whether or not it makes sense to apply the rule (we do not want to
    /// apply a rule if we are sure it will not derive any new information).
    fn is_applicable(&self, state: &State) -> bool;

    /// Applies the rule, possibly modifying the proof state and the rule's
    /// internal state.
    fn apply(&mut self, state: &mut State) -> ();
}

mod material;
pub use material::*;

mod steady;
pub use steady::*;
