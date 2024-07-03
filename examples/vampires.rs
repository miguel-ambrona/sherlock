use chess::{Board, MoveGen};
use sherlock::is_legal;
use std::{collections::HashMap, str::FromStr};

fn main() {
    // the initial position with black to move
    let board = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b - -").unwrap();
    let depth = 15;

    let mut table = HashMap::<Board, u8>::new();
    search(&mut table, &board, depth);

    println!("Vampires up to depth {}: {}", depth, table.len());
}

fn search(table: &mut HashMap<Board, u8>, board: &Board, depth: u8) {
    if depth == 0 {
        return;
    }

    if let Some(stored_depth) = table.get(board) {
        if *stored_depth <= depth {
            return;
        }
    };

    table.insert(*board, depth);

    if !is_legal(board) {
        // this is the mirror image of a vampire!
        if depth <= 1 {
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
