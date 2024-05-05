use std::str::FromStr;

use chess::Board;

pub enum Legal {
    Yes,
    No,
    TBD, // Used for illegal positions that cannot be captured yet
}

#[test]
fn test_legality() {
    use crate::Legal::*;
    [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -", Yes),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq -", TBD),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBNKBNR w - -", No),
        // The following is illegal but only if 0-0 is enableld for White,
        // as promoting on H1 would require only 1 capture
        ("rnbqkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w K -", No),
        ("rn1qkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w K -", TBD),
        ("rnbqkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w - -", Yes),
        // The following is illegal if 0-0-0 is enabled for Black, as
        // no white pawn could have promoted.
        ("r3k3/ppp1p1pp/8/8/8/8/8/R1R1K2R b q -", No),
        ("r3k3/ppp1p1pp/8/8/8/8/8/R1R1K2R b - -", Yes),
        // Without the E7 pawn and the wR on F8 this should be legal
        ("r3kR2/ppp3pp/8/8/8/8/8/R3K2R b q -", Yes),
    ]
    .iter()
    .for_each(|(fen, expected_legal)| {
        let board = Board::from_str(fen).expect("Valid Position");
        let legal = sherlock::is_legal(&board);
        match expected_legal {
            Yes | TBD => assert!(legal),
            No => assert!(!legal),
        }
    })
}
