use std::{collections::HashMap, str::FromStr};

use chess::{BitBoard, Board, ChessMove, Color, MoveGen, Square, EMPTY};
use sherlock::is_legal;

fn main() {
    // the initial position with black to move (the Head Vampire)
    let board = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq -").unwrap();

    let mut table = HashMap::<Board, bool>::new();
    let mut to_be_analyzed = vec![board];
    let mut vampire_images = vec![];

    let mut depth = 1;
    loop {
        if to_be_analyzed.is_empty() {
            break;
        }
        for board in to_be_analyzed.iter() {
            let moves = MoveGen::new_legal(board);
            for m in moves {
                if !preserves_clan(board, &m) {
                    continue;
                }
                let new_board = board.make_move_new(m);
                if table.get(&new_board).is_some() {
                    continue;
                }

                let legal = is_legal(&new_board);
                table.insert(new_board, legal);

                if !legal {
                    vampire_images.push(new_board);
                }
            }
        }
        to_be_analyzed = vampire_images.clone();
        println!("\rVampires of depth {}: {}", depth, vampire_images.len());
        for vampire_image in vampire_images[..5].iter() {
            println!("  D{} {}", depth, vampire_image);
        }

        vampire_images = vec![];
        depth += 1;
    }
}

fn _search(
    table: &mut HashMap<Board, (u8, bool)>,
    vampire_images: &mut Vec<Board>,
    board: &Board,
    depth: u8,
) {
    if depth == 0 {
        return;
    }

    if table.len() % 1000 == 0 {
        dbg!(table.len());
    }

    if let Some((stored_depth, stored_is_legal)) = table.get(board) {
        if *stored_is_legal || *stored_depth >= depth {
            return;
        }
        table.insert(*board, (depth, false));
    } else {
        let legal = is_legal(board);
        table.insert(*board, (depth, legal));

        if legal {
            // we lost the parity invariant, we can stop the search
            return;
        }
    }

    vampire_images.push(*board);

    let moves = MoveGen::new_legal(board);
    for m in moves {
        let new_board = board.make_move_new(m);
        _search(table, vampire_images, &new_board, depth - 1);
    }
}

// We preserve clans, so we are not allow to lose castling rights, except when
// castling.
fn preserves_clan(board: &Board, m: &ChessMove) -> bool {
    let move_bb = BitBoard::from_square(m.get_source()) ^ BitBoard::from_square(m.get_dest());
    if board.castle_rights(Color::White).has_kingside()
        && move_bb & (BitBoard::from_square(Square::E1) | BitBoard::from_square(Square::H1))
            != EMPTY
    {
        return false;
    }
    if board.castle_rights(Color::Black).has_kingside()
        && move_bb & (BitBoard::from_square(Square::E8) | BitBoard::from_square(Square::H8))
            != EMPTY
    {
        return false;
    }
    if board.castle_rights(Color::White).has_queenside()
        && move_bb & (BitBoard::from_square(Square::E1) | BitBoard::from_square(Square::A1))
            != EMPTY
    {
        return false;
    }
    if board.castle_rights(Color::Black).has_queenside()
        && move_bb & (BitBoard::from_square(Square::E8) | BitBoard::from_square(Square::A8))
            != EMPTY
    {
        return false;
    }

    true
}
