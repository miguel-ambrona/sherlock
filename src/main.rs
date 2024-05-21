use std::str::FromStr;

use chess::Board;
use sherlock::analyze;

fn main() {
    // let board = Board::from_str("r1b1kb1r/pp1ppppp/2p5/8/8/2Q5/PP1PPPPP/RN1QKBNR
    // w KQkq -")
    let board = Board::from_str("r2qkb1r/ppp1pppp/8/7n/b2P4/8/PPPPP1PP/RNBQKBNR b KQkq -")
        // let board = Board::from_str("r1bqkb1r/1ppppppp/8/2P5/8/8/PPPPP1PP/R1BQKB1R w KQkq -")
        .expect("Valid Position");
    let analysis = analyze(&board);
    println!("{}", analysis)
}
