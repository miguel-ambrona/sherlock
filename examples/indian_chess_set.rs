//! This example includes a problem by Raymond M. Smullyan, from the book
//! "The Chess Mysteries of Sherlock Holmes".
//! See Chapter "The Mystery of the Indiant Chess Set".

use std::str::FromStr;

use chess::Board;
use sherlock::is_legal;

fn main() {
    let board_from_white =
        Board::from_str("r1b1kb1r/pppppppp/2N5/5n2/6N1/2n5/PPPPPPPP/1RBK1B1R w - -").unwrap();

    assert!(!is_legal(&board_from_white));

    let board_from_black =
        Board::from_str("r1b1kbr1/pppppppp/5N2/1n6/2N5/5n2/PPPPPPPP/R1BK1B1R b - -").unwrap();

    assert!(is_legal(&board_from_black));
}
