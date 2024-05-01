use chess::Board;

use crate::{
    analysis::Analysis,
    rules::{
        CapturesBoundsRule, DestiniesRule, MaterialRule, OriginsRule, RefineOriginsRule,
        RouteFromOriginsRule, RouteToDestiniesRule, Rule, SteadyRule,
    },
};

/// Initialize all the available rules.
fn init_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(MaterialRule::new()),
        Box::new(SteadyRule::new()),
        Box::new(OriginsRule::new()),
        Box::new(RefineOriginsRule::new()),
        Box::new(DestiniesRule::new()),
        Box::new(CapturesBoundsRule::new()),
        Box::new(RouteFromOriginsRule::new()),
        Box::new(RouteToDestiniesRule::new()),
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
    let mut rules = init_rules();
    let mut analysis = Analysis::new(board);
    loop {
        let mut progress = false;
        for rule in rules.iter_mut() {
            if rule.is_applicable(&analysis) && analysis.illegal.is_none() {
                rule.update(&analysis);
                progress |= rule.apply(&mut analysis);
            }
        }
        if !progress || analysis.illegal.is_some() {
            break;
        }
    }
    analysis.illegal != Some(true)
}
