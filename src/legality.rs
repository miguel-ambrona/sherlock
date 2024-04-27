use chess::Board;

use crate::{
    rules::{CapturesBoundsRule, MaterialRule, OriginsRule, RefineOriginsRule, Rule, SteadyRule},
    state::State,
};

/// Initialize all the available rules on the given board.
fn init_rules(board: &Board) -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(MaterialRule::new(board)),
        Box::new(SteadyRule::new(board)),
        Box::new(OriginsRule::new(board)),
        Box::new(RefineOriginsRule::new(board)),
        Box::new(CapturesBoundsRule::new(board)),
    ]
}

/// Checks whether the given `Board` is *legal*, i.e. reachable from the
/// starting chess position via a sequence of legal moves.
///
/// This is a semi-decision procedure in the sense that:
///  - If the output is `false`, the given position is *definitely illegal*.
///  - If the output is `true`, the given position is probably legal, but *it
///    might not be legal* if it escapes the current logic.
///
/// ```
/// use chess::Board;
/// use sherlock::is_legal;
///
/// let board = Board::default();
/// assert!(is_legal(&board));
/// ```
pub fn is_legal(board: &Board) -> bool {
    let mut rules = init_rules(board);
    let mut state = State::new(board);
    loop {
        state.progress = false;
        for rule in rules.iter_mut() {
            if rule.is_applicable(&state) && state.illegal.is_none() {
                rule.apply(&mut state);
            }
        }
        if !state.progress || state.illegal.is_some() {
            break;
        }
    }
    state.illegal != Some(true)
}
