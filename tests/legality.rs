use std::str::FromStr;

use chess::Board;

pub enum Legality {
    Ok,
    Illegal,
    TBD, // Used for illegal positions that cannot be captured yet
}

#[test]
fn test_legality() {
    use crate::Legality::*;
    [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -", Ok),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq -", TBD),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBNKBNR w - -", Illegal),
    ]
    .iter()
    .for_each(|(fen, expected_result)| {
        let board = Board::from_str(fen).expect("Valid Position");
        let legal = sherlock::is_legal(&board);
        match expected_result {
            Ok | TBD => assert!(legal),
            Illegal => assert!(!legal),
        }
    })
}
