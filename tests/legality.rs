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
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq -", No),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBNKBNR w - -", No),
        // the following is illegal but only if 0-0 is enableld for White,
        // as promoting on H1 would require only 1 capture
        ("rnbqkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w K -", No),
        ("rn1qkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w K -", TBD),
        ("rnbqkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w - -", Yes),
        // the following is illegal if 0-0-0 is enabled for Black, as
        // no white pawn could have promoted
        ("r3k3/ppp1p1pp/8/8/8/8/8/R1R1K2R b q -", No),
        ("r3k3/ppp1p1pp/8/8/8/8/8/R1R1K2R b - -", Yes),
        // without the E7 pawn and the wR on F8 this should be legal
        ("r3kR2/ppp3pp/8/8/8/8/8/R3K2R b q -", Yes),
        // parity tests
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R b Qq -", Yes),
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R w Qq -", No),
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R w q -", Yes),
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R w Q -", Yes),
        ("rnb1kb2/pppppppr/7p/8/8/P5PP/1PPPP1PR/RNB1KBN1 w Qq -", No),
        ("rnb1kb2/pppppppr/7p/8/8/P5PP/1PPPP1PR/RNB1KBN1 b Qq -", Yes),
        ("rnb1kb2/pppppppr/7p/8/8/P5PP/1PPPP1PR/RNB1KB2 w Qq -", Yes),
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
