use std::collections::HashMap;

use chess::Board;

use crate::{analysis::Analysis, rules::*, Legality::Illegal, RetractableBoard, RetractionGen};

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
        Box::new(MobilityRule::new()),
        Box::new(RouteFromOriginsRule::new()),
        Box::new(RouteToReachable::new()),
        Box::new(MissingRule::new()),
        Box::new(CapturesRule::new()),
        Box::new(TombsRule::new()),
        Box::new(ParityRule::new()),
    ]
}

/// Analyzes the legality of the position using all the existing rules.
/// Returns a report containing all the information derived about the
/// position.
/// ```
/// use chess::{Board, Square};
/// use sherlock::{analyze, RetractableBoard};
///
/// let analysis = analyze(&RetractableBoard::default());
/// assert_eq!(analysis.is_steady(Square::D1), true);
/// assert_eq!(analysis.is_steady(Square::B1), false);
/// ```
pub fn analyze(board: &RetractableBoard) -> Analysis {
    let mut rules = init_rules();
    let mut analysis = Analysis::new(board);
    loop {
        let mut progress = false;
        for rule in rules.iter_mut() {
            if rule.is_applicable(&analysis) && analysis.result.is_none() {
                println!("{:?}", rule);
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

/// If the position is illegal, it returns `false`. Otherwise, if the position
/// is [limited in retractions](RetractionGen::is_limited_in_retractions), it
/// retracts it in all possible ways and recurses.
fn is_retractable(table: &mut HashMap<RetractableBoard, bool>, board: &RetractableBoard) -> bool {
    if let Some(b) = table.get(board) {
        return *b;
    };

    let analysis = analyze(board);
    if analysis.result == Some(Illegal) {
        return false;
    } else if !RetractionGen::is_limited_in_retractions(board) {
        return true;
    }

    // add the position to the table as "false" to avoid infinite-loops, we will
    // correct this when the analysis is over
    table.insert(*board, false);
    let mut res = false;

    let mut retractions = RetractionGen::new_legal(board);
    retractions.refine_iterator(&analysis);
    for r in retractions {
        let new_board = board.make_retraction_new(r);
        if is_retractable(table, &new_board) {
            res = true;
            break;
        }
    }

    if res {
        table.insert(*board, res);
    }
    res
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
    let mut table = HashMap::<RetractableBoard, bool>::new();
    is_retractable(&mut table, &(*board).into())
}
