use std::str::FromStr;

use chess::Board;
use sherlock::analyze;

fn main() {
    let board = Board::from_str("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR b - -")
        .expect("Valid Position");

    let analysis = analyze(&board.into());
    println!("{analysis}");
}
