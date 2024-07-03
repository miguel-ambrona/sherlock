use std::{collections::HashMap, str::FromStr};

use chess::{Board, MoveGen};
use sherlock::is_legal;

const MAX_DEPTH: u8 = 15;

fn main() {
    // the initial position with black to move (the Head Vampire)
    let board = Board::from_str("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR b - -").unwrap();

    let mut table = HashMap::<Board, u8>::new();
    for depth in 0..MAX_DEPTH {
        println!("Analyzing depth: {}", depth);
        search(&mut table, &board, depth);
    }

    println!("Vampires up to depth {}: {}", MAX_DEPTH, table.len());
}

fn search(table: &mut HashMap<Board, u8>, board: &Board, depth: u8) {
    if depth == 0 {
        return;
    }

    if let Some(stored_depth) = table.get(board) {
        if *stored_depth >= depth {
            return;
        }
    };

    table.insert(*board, depth);

    if !is_legal(board) {
        // this is the mirror image of a vampire!
        if depth >= 15 {
            println!("{}, {}", depth, board);
        }
    } else {
        // we lost the parity invariant, we can stop the search
        return;
    }

    let moves = MoveGen::new_legal(board);
    for m in moves {
        let new_board = board.make_move_new(m);
        search(table, &new_board, depth - 1);
    }
}
