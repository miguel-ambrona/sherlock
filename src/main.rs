use std::str::FromStr;

use chess::Board;
use sherlock::is_legal;

fn main() {
    let board = Board::from_str("rnbqkbnr/ppppppp1/8/8/8/8/1PPPPP1P/RNBQKBNR w Kkq -")
        .expect("Valid Position");
    println!("is_legal: {}", is_legal(&board))
}
