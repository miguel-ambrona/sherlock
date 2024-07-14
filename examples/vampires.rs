use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, Write},
    str::FromStr,
};

use chess::{BitBoard, Board, ChessMove, MoveGen, EMPTY};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sherlock::{is_legal, RetractableBoard, RetractionGen};

// The squares including squares affecting the clan's castling rights.
const CLAN_BB: BitBoard = BitBoard(10448351135499550865); // KQkq

// We preserve clans if we do not lose castling rights (except by castling).
// However, castling moves will be studyied in a post-processing, not here.
fn preserves_clan(m: &ChessMove) -> bool {
    let move_bb = BitBoard::from_square(m.get_source()) ^ BitBoard::from_square(m.get_dest());
    move_bb & CLAN_BB == EMPTY
}

// Quick test to see if the position is legal.
// It returns `None` if the test was not conclusive.
// fn quick_legality_test(table: &HashSet<Board>, board: &Board) -> Option<bool>
// {     // If moving forwards we reach an illegal position, we are illegal.
//     for m in MoveGen::new_legal(board) {
//         let new_board = board.make_move_new(m);
//         if table.contains(&new_board) {
//             return Some(false);
//         }
//     }

//     // // If moving backwards we reach a legal position, we are legal.
//     // let retractable_board: RetractableBoard = (*board).into();
//     // for r in RetractionGen::new_legal(&retractable_board) {
//     //     let new_board = retractable_board.make_retraction_new(r);
//     //     let board = Board::from_str(format!("{}",
// new_board).as_str()).unwrap();     //     if table.get(&board) == Some(&true)
// {     //         return Some(true);
//     //     }
//     // }

//     None
// }

fn main() {
    let loading = true;

    let mut vampires_file = File::create("vampires-KQkq-22.txt").unwrap();

    let mut table = HashSet::<Board>::new();
    let mut previous_vampire_images = HashSet::<Board>::new();
    let mut current_vampire_images = HashSet::<Board>::new();
    let mut depth = 1;

    let last_depth = 20;
    let final_depth = 22;

    // load existing file in memory
    if loading {
        let file = File::open("vampires-KQkq-20.txt").unwrap();

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let mut fen = "".to_owned();
            let l = line.unwrap();
            for (i, token) in l.split_ascii_whitespace().enumerate() {
                if i == 0 {
                    depth = token[1..].parse::<i32>().unwrap();
                }
                if i > 1 {
                    fen.push_str(token);
                    fen.push(' ');
                }
            }
            let board = Board::from_str(fen.as_str()).unwrap();

            table.insert(board);

            if depth == last_depth {
                previous_vampire_images.insert(board);
            }
        }
        depth += 1;

        println!("Loaded {} positions", table.len());
        println!(
            "Analyzing {} positions of {}-depth on depth {}",
            previous_vampire_images.len(),
            depth - 1,
            depth
        );
    } else {
        // let mut humans_file = File::create("humans-KQkq.txt").unwrap();

        // the initial position with black to move (the Head Vampire's image)
        let board =
            Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq -").unwrap();

        vampires_file
            .write_fmt(format_args!("D0 P32 {}\n", board))
            .unwrap();

        previous_vampire_images.insert(board);
        table.insert(board);
    }

    loop {
        if depth > final_depth {
            break;
        }

        if previous_vampire_images.is_empty() {
            break;
        }

        let mut to_be_analyzed = HashSet::<Board>::new();

        for board in previous_vampire_images.iter() {
            let moves = MoveGen::new_legal(board);
            for m in moves {
                if !preserves_clan(&m) {
                    continue;
                }
                let new_board = board.make_move_new(m);
                if table.get(&new_board).is_some() {
                    continue;
                }

                to_be_analyzed.insert(new_board);
            }
        }

        let nb_cores = 16;
        let cores: Vec<usize> = (0..nb_cores).collect();

        let nb_to_be_analyzed = to_be_analyzed.len();
        let nb_boards_per_core = nb_to_be_analyzed.div_ceil(nb_cores);
        let to_be_analyzed: Vec<Board> = to_be_analyzed.iter().cloned().collect();

        let local_tables = cores
            .par_iter()
            .map(|core_idx| {
                let mut local_table = HashSet::<Board>::new();
                let start = core_idx * nb_boards_per_core;
                let end = min((core_idx + 1) * nb_boards_per_core, nb_to_be_analyzed);

                if start <= end {
                    for board in to_be_analyzed[start..end].iter() {
                        // let legal = match quick_legality_test(&table, board) {
                        // None => is_legal(board),
                        // Some(res) => res,
                        // };
                        let legal = is_legal(board);
                        if !legal {
                            local_table.insert(*board);
                        }
                    }
                }

                local_table
            })
            .collect::<Vec<_>>();

        for local_table in local_tables.iter() {
            table.extend(local_table.iter());
            for board in local_table {
                // if !legal {
                current_vampire_images.insert(*board);
                let n = board.combined().popcnt();
                vampires_file
                    .write_fmt(format_args!("D{} P{} {}\n", depth, n, board))
                    .unwrap();
                // }
                //  else {
                // humans_file.write_fmt(format_args!("{}\n", board)).unwrap();
                // }
            }
        }

        previous_vampire_images = current_vampire_images.clone();
        println!(
            "\rVampires of depth {}: {}",
            depth,
            current_vampire_images.len()
        );

        current_vampire_images = HashSet::<Board>::new();
        depth += 1;
    }
}
