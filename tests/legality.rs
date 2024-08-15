use std::str::FromStr;

use chess::Board;

pub enum Legality {
    Legal,
    Illegal,
    TBD, // Used for illegal positions that cannot be captured yet
}

fn test_legality(positions: &[(&str, Legality)]) {
    use crate::Legality::*;
    positions.iter().for_each(|(fen, expected_legal)| {
        let board = Board::from_str(fen).expect("Valid Position");
        let legal = sherlock::is_legal(&board);
        match expected_legal {
            Legal | TBD => assert!(legal),
            Illegal => assert!(!legal),
        }
    })
}

#[test]
fn test_legality_misc() {
    use crate::Legality::*;
    #[rustfmt::skip]
    let positions = [
        // simple parity examples on the starting array
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -", Legal),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq -", Illegal),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBNKBNR w - -", Illegal),
        ("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR b - -", Illegal),

        // the following is illegal but only if 0-0 is enableld for White,
        // as promoting on H1 would require only 1 capture
        ("rnbqkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w K -", Illegal),
        ("rn1qkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w K -", TBD),
        ("rnbqkbnr/pppppp1p/8/3b4/8/6P1/PPPPPP2/RNBQK1NR w - -", Legal),

        // the following is illegal if 0-0-0 is enabled for Black, as
        // no white pawn could have promoted
        ("r3k3/ppp1p1pp/8/8/8/8/8/R1R1K2R b q -", Illegal),
        ("r3k3/ppp1p1pp/8/8/8/8/8/R1R1K2R b - -", Legal),
        // without the E7 pawn and the wR on F8 this should be legal
        ("r3kR2/ppp3pp/8/8/8/8/8/R3K2R b q -", Legal),

        // parity tests
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R b Qq -", Legal),
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R w Qq -", Illegal),
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R w q -", Legal),
        ("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R w Q -", Legal),
        ("rnb1kb2/pppppppr/7p/8/8/P5PP/1PPPP1PR/RNB1KBN1 w Qq -", Illegal),
        ("rnb1kb2/pppppppr/7p/8/8/P5PP/1PPPP1PR/RNB1KBN1 b Qq -", Legal),
        ("rnb1kb2/pppppppr/7p/8/8/P5PP/1PPPP1PR/RNB1KB2 w Qq -", Legal),
        ("Nrq1kb1r/pppppppp/1N6/8/1P6/4n1n1/1PPPPPPP/R1BQKB1R b KQk -", Illegal),
        // the following is illegal due to parity, but after f7-f6 the bRg8
        // may have triangulated, as the bNh8 is not blocked anymore
        ("rnbqkbrn/ppppppp1/6p1/8/8/8/PPPPPPPP/RNBQKB1R b - -", TBD),
        ("rnbqkbrn/ppppp1p1/5pp1/8/8/8/PPPPPPPP/RNBQKB1R w - -", Legal),


        // the following should be illegal, as the promoted white knight 
        // cannot have possibly left A8
        ("1nbqkbnr/1ppppppp/1p6/3rN3/8/8/1PPPPPPP/R1BQKBNR b - -", TBD),
        ("Nnbqkbnr/1ppppppp/1p6/3r4/8/8/1PPPPPPP/R1BQKBNR b - -", Legal),
    ];
    test_legality(&positions)
}

#[test]
#[ignore]
fn test_legality_slow() {
    use crate::Legality::*;
    #[rustfmt::skip]
    let positions = [
        // a castling vampire-image that becomes legal on minor changes
        ("r1b1k2r/1pppppp1/7B/p7/1N6/1PP5/NPP1PPPP/2KR1B1R w kq -", Illegal),
        ("r1b1k2r/1pppppp1/7B/p7/1N6/1PP5/NPP1PPPP/2KR1BR1 w kq -", Legal),
        ("r1b1k2r/1pppppp1/7B/p7/1N6/1PP5/NPP1PPPP/2KR1B1R b kq -", Legal),
        ("r1b1k2r/1pppppp1/7B/p7/1N6/1PP5/NPP1PPPP/2KR1B1R w - -", Legal),
    ];
    test_legality(&positions)
}
