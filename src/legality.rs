use chess::Board;

use crate::rules::{Rule, State, SteadyRule};

/// Initialize on the given board all the available rules.
fn init_rules(board: &Board) -> Vec<Box<dyn Rule>> {
    let steady_rule = SteadyRule::new(board);
    vec![Box::new(steady_rule)]
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
            if rule.is_applicable(&state) {
                rule.apply(&mut state);
            }
        }
        if !state.progress {
            break;
        }
    }
    state.illegal != Some(false)
}
