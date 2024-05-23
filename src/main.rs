use std::str::FromStr;

use chess::Board;
use sherlock::analyze;

fn main() {
    let board = Board::from_str("rnb1kb2/1pppp1pr/p4p1p/8/8/P5PP/1PPPP1PR/RNB1KBN1 b Qq -")
        .expect("Valid Position");
    let analysis = analyze(&board);
    println!("{}", analysis)
}
