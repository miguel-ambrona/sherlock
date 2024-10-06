use std::str::FromStr;

use chess::Board;
use sherlock::analyze;

fn main() {
    let board = Board::from_str("r2qkbnr/ppp1pppp/2p5/8/8/5PP1/PPPP2PP/RNBQK1NR w KQkq -")
        .expect("Valid Position");

    let a = analyze(&board.into());
    println!("{a}");
}
