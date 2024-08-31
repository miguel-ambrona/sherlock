use std::{collections::HashMap, str::FromStr};

use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, Square, EMPTY};

use sherlock::is_legal;

// The squares including squares affecting the clan's castling rights.
const CLAN_BB: BitBoard = BitBoard(10448351135499550865); // KQkq

// We preserve clans if we do not lose castling rights (except by castling).
// However, castling moves will be studyied in a post-processing, not here.
fn preserves_clan(m: &ChessMove) -> bool {
    let move_bb = BitBoard::from_square(m.get_source()) ^ BitBoard::from_square(m.get_dest());
    move_bb & CLAN_BB == EMPTY
}

fn analyze(vampire_images: &mut HashMap<Board, u8>, board: &Board, depth: u8) {
    if depth == 0 {
        return;
    }

    let mut found = false;
    if let Some(d) = vampire_images.get(board) {
        if *d >= depth {
            return;
        }
        found = true;
    }

    vampire_images.insert(*board, depth);

    if !found {
        for m in MoveGen::new_legal(board) {
            if preserves_clan(&m)
                && BitBoard::from_square(m.get_source()) & board.pieces(Piece::Pawn) != EMPTY
                && (m.get_source() != Square::A2 || m.get_dest() != Square::A3)
                && (m.get_source() != Square::A2 || m.get_dest() != Square::B3)
                && (m.get_source() != Square::F2 || m.get_dest() != Square::E3)
                && (m.get_source() != Square::F2 || m.get_dest() != Square::F3)
                && (m.get_source() != Square::F2 || m.get_dest() != Square::G3)
                && (m.get_source() != Square::H2 || m.get_dest() != Square::G3)
                && (m.get_source() != Square::H2 || m.get_dest() != Square::H3)
                && (m.get_source() != Square::A7 || m.get_dest() != Square::A6)
                && (m.get_source() != Square::A7 || m.get_dest() != Square::B6)
                && (m.get_source() != Square::F7 || m.get_dest() != Square::E6)
                && (m.get_source() != Square::F7 || m.get_dest() != Square::F6)
                && (m.get_source() != Square::F7 || m.get_dest() != Square::G6)
                && (m.get_source() != Square::H7 || m.get_dest() != Square::G6)
                && (m.get_source() != Square::H7 || m.get_dest() != Square::H6)
                && (m.get_source() != Square::B3 || m.get_dest() != Square::C4)
            {
                let new_board = board.make_move_new(m);
                if !is_legal(&new_board) {
                    println!("{}", new_board);
                }
            }
        }
    }

    for m in MoveGen::new_legal(board) {
        if preserves_clan(&m)
            && BitBoard::from_square(m.get_source()) & board.pieces(Piece::Knight) != EMPTY
            && BitBoard::from_square(m.get_dest()) & board.combined() == EMPTY
        {
            let new_board = board.make_move_new(m);
            analyze(vampire_images, &new_board, depth - 1)
        }
    }
}

fn main() {
    const MAX_DEPTH: u8 = 50;
    let mut vampire_images = HashMap::<Board, u8>::new();

    let board =
        Board::from_str("rnbqkb1r/pppppppp/8/8/8/1P6/1PPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
    vampire_images.insert(board, 0);

    let mut count = 0;

    for depth in 1..=MAX_DEPTH {
        analyze(&mut vampire_images, &board, depth);
        println!("{} {}", depth, vampire_images.len() - count);
        if count == vampire_images.len() {
            break;
        }
        count = vampire_images.len();
    }

    println!("Total number of vampires: {}", vampire_images.len());
}
