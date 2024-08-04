use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    str::FromStr,
};

use chess::{BitBoard, Board, ChessMove, MoveGen, EMPTY};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sherlock::{is_legal, RetractionGen};

// The squares including squares affecting the clan's castling rights.
const CLAN_BB: BitBoard = BitBoard(10448351135499550865); // KQkq

// We preserve clans if we do not lose castling rights (except by castling).
// However, castling moves will be studyied in a post-processing, not here.
fn preserves_clan(m: &ChessMove) -> bool {
    let move_bb = BitBoard::from_square(m.get_source()) ^ BitBoard::from_square(m.get_dest());
    move_bb & CLAN_BB == EMPTY
}

fn analyze(
    vampire_images: &mut HashMap<Board, u8>,
    limited_images: &mut HashSet<Board>,
    board: &Board,
    depth: u8,
) {
    if depth == 0 {
        return;
    }

    let mut moves = vec![];
    for m in MoveGen::new_legal(board) {
        if preserves_clan(&m) {
            moves.push(m);
        }
    }

    let illegal_boards = moves
        .par_iter()
        .map(|m| {
            let new_board = board.make_move_new(*m);
            let fetched = vampire_images.get(&new_board);

            // if we are not at the border (depth = 1), an unknown position must be legal
            if fetched.is_none() && depth > 1 {
                return None;
            }

            // do not analyze positions that were analyzed at a higher depth
            if let Some(analyzed_depth) = fetched {
                if *analyzed_depth >= depth {
                    return None;
                }
            }

            // if it is fetched, we already know it is illegal
            if fetched.is_some() {
                return Some(new_board);
            }

            // we work under the assumption that the invariant is preserved, this allows us
            // to skip some legality checks

            // do not continue analyzing legal positions (they are not vampire images)
            if is_legal(&new_board) {
                return None;
            }

            Some(new_board)
        })
        .collect::<Vec<_>>();

    for new_board in illegal_boards.iter().flatten() {
        vampire_images.insert(*new_board, depth);

        if RetractionGen::is_limited_in_retractions(&(*new_board).into())
            && *new_board.checkers() == EMPTY
        {
            // if it is the first time we see this position.
            if depth == 1 {
                limited_images.insert(*new_board);
            }
        } else {
            analyze(vampire_images, limited_images, new_board, depth - 1)
        }
    }
}

fn main() {
    const MAX_DEPTH: u8 = 8;
    let mut vampires_file = File::create("interesting-KQkq.txt").unwrap();
    let mut vampire_images = HashMap::<Board, u8>::new();
    let mut limited_images = HashSet::<Board>::new();

    // the initial position with black to move (the Head Vampire's image)
    let board = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq -").unwrap();
    vampire_images.insert(board, 0);

    let mut count = 1;

    for depth in 1..=MAX_DEPTH {
        analyze(&mut vampire_images, &mut limited_images, &board, depth);
        println!("{} {}", depth, vampire_images.len() - count);
        count = vampire_images.len();
    }

    println!("Vampires so far: {}", count);

    loop {
        if limited_images.is_empty() {
            break;
        }

        println!(
            "{} limited in retractions to be analyzed...",
            limited_images.len()
        );

        let mut limited_frontier = HashSet::<Board>::new();
        for board in limited_images {
            let mut moves = vec![];
            for m in MoveGen::new_legal(&board) {
                if preserves_clan(&m) {
                    moves.push(m);
                }
            }

            let illegal_boards = moves
                .par_iter()
                .map(|m| {
                    let new_board = board.make_move_new(*m);

                    if vampire_images.get(&new_board).is_some() {
                        return None;
                    }

                    // TODO: Remove me from the final search.
                    if !RetractionGen::is_limited_in_retractions(&(new_board).into()) {
                        return None;
                    }

                    if is_legal(&new_board) {
                        return None;
                    }

                    Some(new_board)
                })
                .collect::<Vec<_>>();

            for new_board in illegal_boards.iter().flatten() {
                vampire_images.insert(*new_board, 0);
                limited_frontier.insert(*new_board);
                vampires_file
                    .write_fmt(format_args!("{}\n", new_board))
                    .unwrap();
            }
        }

        limited_images = limited_frontier;
    }

    println!("Total number of vampires: {}", vampire_images.len());
}
