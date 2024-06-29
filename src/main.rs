use std::str::FromStr;

use chess::Board;
use sherlock::is_legal;

fn main() {
    let board = Board::from_str("Nrq1kb1r/pppppppp/1N6/8/1P6/4n1n1/1PPPPPPP/R1BQKB1R b KQk -")
        .expect("Valid Position");

    let legal = is_legal(&board);
    dbg!(legal);
}
