use chess::Board;

use crate::{analysis::Analysis, rules::*, Legality::Illegal};

/// Initialize all the available rules.
fn init_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(MaterialRule::new()),
        Box::new(SteadyRule::new()),
        Box::new(OriginsRule::new()),
        Box::new(RefineOriginsRule::new()),
        Box::new(DestiniesRule::new()),
        Box::new(SteadyMobilityRule::new()),
        Box::new(PawnOn3rdRankRule::new()),
        Box::new(CapturesBoundsRule::new()),
        Box::new(RouteFromOriginsRule::new()),
        Box::new(RouteToReachable::new()),
    ]
}

/// Analyzes the legality of the position using all the existing rules.
/// Returns a report containing all the information derived about the
/// position.
/// ```
/// use chess::{Board, Square};
/// use sherlock::analyze;
///
/// let analysis = analyze(&Board::default());
/// assert_eq!(analysis.is_steady(Square::D1), true);
/// assert_eq!(analysis.is_steady(Square::B1), false);
/// ```
pub fn analyze(board: &Board) -> Analysis {
    let mut rules = init_rules();
    let mut analysis = Analysis::new(board);
    loop {
        let mut progress = false;
        for rule in rules.iter_mut() {
            if rule.is_applicable(&analysis) && analysis.result.is_none() {
                rule.update(&analysis);
                progress |= rule.apply(&mut analysis);
            }
        }
        if !progress || analysis.result.is_some() {
            break;
        }
    }
    analysis
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
    let analysis = analyze(board);
    analysis.result != Some(Illegal)
}
