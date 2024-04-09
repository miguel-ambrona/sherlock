use std::str::FromStr;

use chess::Board;
use sherlock;

pub enum Result {
    Legal,
    Illegal,
    TBD, // Used for illegal positions that cannot be captured yet
}

#[test]
fn test_legality() {
    use crate::Result::*;
    [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNQNKBNR w - -", Legal),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNQNKBNR b - -", TBD),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBNKBNR w - -", TBD),
    ]
    .iter()
    .for_each(|(fen, expected_result)| {
        let board = Board::from_str(fen).expect("Valid Position");
        let legal = sherlock::is_legal(&board);
        match expected_result {
            Legal | TBD => assert!(legal),
            Illegal => assert!(!legal),
        }
    })
}
