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
        ("r1bnkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -", Legal),
        ("rnbqkbnr/1pppppp1/1N5p/1p6/8/8/PPPPPPPP/R1BQKB1R w KQkq -", Legal),

        ("r1bqk2r/1pppp1p1/8/5pN1/2Q4p/PP5n/pBPPPPPP/N3K1nR w Kkq -", Illegal),

        // the following is illegal due to parity, but after f7-f6 the bRg8
        // may have triangulated, as the bNh8 is not blocked anymore
        ("rnbqkbrn/ppppppp1/6p1/8/8/8/PPPPPPPP/RNBQKB1R b - -", TBD),
        ("rnbqkbrn/ppppp1p1/5pp1/8/8/8/PPPPPPPP/RNBQKB1R w - -", Legal),


        // the following should be illegal, as the promoted white knight 
        // cannot have possibly left A8
        ("1nbqkbnr/1ppppppp/1p6/3rN3/8/8/1PPPPPPP/R1BQKBNR b Q -", Illegal),
        ("1nbqkbnr/1ppppppp/1p6/3rN3/8/8/1PPPPPPP/R1BQKBNR b - -", Legal), // wRa1 -> b6
        ("Nnbqkbnr/1ppppppp/1p6/3r4/8/8/1PPPPPPP/R1BQKBNR b Q -", Legal),

        // misc
        ("rnbqkBnr/pppppp2/6p1/7p/8/3P4/PPP1PPPP/RN1QKBNR w KQkq -", Legal),
        ("rn2k1nr/3p4/1P2Pp1P/P1P3P1/4p3/ppp2Ppp/3P4/RN1QK1NR w - -", Illegal),
        ("rn2k1nr/3p4/1P2Pp1P/P1P3P1/4p3/ppp2Ppp/3P4/RN2K1NR w - -", Legal),
        ("rnbk2nr/PP3PPP/1P3P2/8/6p1/ppppppp1/8/RN2K1NR w - -", Illegal),
        ("rn1k2nr/PP3PPP/1P3P2/8/6p1/ppppppp1/8/RN2K1NR w - -", TBD),
        ("rn1k2n1/PP3PPP/1P3P2/8/6p1/ppppppp1/8/RN2K1NR w - -", Legal),


        // github issues #36 - #44
        ("rnbqkbnr/ppp2ppp/4p3/3P4/3p4/4P3/PPP2PPP/RNBQKBNR b - -", Illegal),
        ("7b/6p1/3k4/8/8/2K5/8/8 w - -", Illegal),
        ("4k2r/pp1pp1p1/1np2n1p/qr3p2/2PP1N1P/B1PQP1PB/P2KNPR1/R7 b - -", Illegal),
        ("rn2kb2/pp5p/2bppp2/2p2p2/2Nq4/5N2/PPPPPPPP/2B1KB2 b - -", Illegal),
        ("4k3/8/8/8/8/4P3/1K1PRP2/4b3 b - -", Illegal),
        ("4k3/8/8/8/8/1P6/bPP5/1b2K3 b - -", Illegal),
        ("2b4r/p2p1pp1/2p1r2p/1p2q1k1/P1Nnp3/8/1PPPPPPP/R1B1KQ1R w - -", Illegal),
        ("3r3r/p2p1p1p/b1p2kp1/1p1npq1n/P7/8/1PPPPPPP/RNB2RQK w - -", Illegal),
        ("3r3r/p2p1p1p/b1p2kp1/1p1npq1n/P7/8/1PPPPPPP/RNB1QR1K w - -", Legal),
        ("8/8/8/8/8/P3P2P/1PPP1PP1/2k2K2 w - -", Illegal),
        ("3k1b1K/4ppp1/7p/8/8/8/8/8 w - -", Illegal),
        ("8/8/8/8/8/6P1/6P1/R3K2k w Q -", Illegal),

        // cages - credit to Theodore Hwa:
        // https://github.com/hwatheod/retractor-python/blob/main/doc/cages.pdf
        ("4k1b1/5pp1/6p1/8/8/8/8/4K3 b - -", Illegal),
        ("4k3/8/8/8/8/5P2/4PrPP/7K w - -", Legal),
        ("4k3/8/8/8/8/6P1/4PPrP/7K w - -", TBD),
        ("Knrk4/BpppRp2/1p2p3/8/8/8/8/8 b - -", Illegal),
        ("Knrk4/BpppRp2/1p2p3/8/8/8/8/8 w - -", TBD),
        ("KBrk4/1pppRp2/1p2p3/8/8/8/8/8 b - -", TBD),

        // Smullyan
        ("2nb3K/pkPRp1p1/p2p4/P1p5/1Pp4Q/2PP2P1/4P2P/n7 w - -", Illegal),
        ("2nb3K/pkPRp1p1/p2p4/P1p5/1Pp4B/2PP2P1/4P2P/n7 w - -", Legal),
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
