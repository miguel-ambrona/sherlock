//! Tombs rule.
//!
//! We make sure that all known capturing squares can be reached by an opponent
//! piece to be captured. This allows us to deduce new information about e.g.
//! the destinies of a pieces.

use chess::{BitBoard, Color, Piece, Square, ALL_COLORS, ALL_FILES, ALL_RANKS, EMPTY};

use super::{Analysis, Rule, COLOR_ORIGINS};
use crate::{utils::find_k_group, Legality};

#[derive(Debug)]
pub struct TombsRule {
    destinies_counter: usize,
    missing_counter: usize,
    captures_counter: usize,
}

impl Rule for TombsRule {
    fn new() -> Self {
        TombsRule {
            destinies_counter: 0,
            missing_counter: 0,
            captures_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.destinies_counter = analysis.destinies.counter();
        self.missing_counter = analysis.missing.counter();
        self.captures_counter = analysis.captures.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.destinies_counter != analysis.destinies.counter()
            || self.missing_counter != analysis.missing.counter()
            || self.captures_counter != analysis.captures.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for color in ALL_COLORS {
            let mut tombs = vec![];
            // we init an array of length 64 in order to be able to call k-groups, the i-th
            // position in the array represents the candidate opponent pieces that may have
            // been captured in the i-th tomb
            let mut captured_candidates = [EMPTY; 64];
            for origin in COLOR_ORIGINS[color.to_index()] {
                for tomb in analysis.captures(origin) {
                    captured_candidates[tombs.len()] =
                        missing_with_target_as_candidate_destiny(analysis, !color, tomb);
                    tombs.push(BitBoard::from_square(tomb));
                }

                // if we do not have forced-captures information, but the piece is a pawn and we
                // know it has performed some captures, we may still include a set of tombs, if
                // its destiny is clear, it is defintely not missing and it finishes as a pawn
                if analysis.nb_captures_lower_bound(origin) > 0
                    && analysis.captures(origin) == EMPTY
                    && analysis.destinies(origin).popcnt() == 1
                    && analysis.is_definitely_on_the_board(origin)
                    && analysis
                        .board
                        .piece_on(analysis.destinies(origin).to_square())
                        == Some(Piece::Pawn)
                {
                    let destiny = analysis.destinies(origin).to_square();

                    let file_origin = origin.get_file().to_index();
                    let file_destiny = destiny.get_file().to_index();
                    let file_diff = (file_destiny as i32 - file_origin as i32).abs();

                    let rank_origin = origin.get_rank().to_index();
                    let rank_destiny = destiny.get_rank().to_index();
                    let rank_diff = (rank_destiny as i32 - rank_origin as i32).abs();

                    let vertical_dir = if color == Color::White { 1 } else { -1 };
                    let horizontal_dir = if file_origin < file_destiny { 1 } else { -1 };

                    // we only continue if the capturing horizontal direction is unique (i.e., it is
                    // not possible to reach the destiny by a capture to the right followed by a
                    // sequence of captures to the left); this is guaranteed if an upper bound on
                    // the number of captures prevents so, or if the file difference greater than or
                    // equal to the rank difference minus one
                    if analysis.nb_captures_upper_bound(origin) - file_diff <= 1
                        || file_diff >= rank_diff - 1
                    {
                        // for every file in between, add a window of potential capturing squares in
                        // that file to the set of tombs
                        let window_size = (file_diff - rank_diff).unsigned_abs() as usize;
                        for i in 1..=file_diff as usize {
                            let mut tomb_candidates = EMPTY;
                            let mut tomb_squares = EMPTY;

                            for j in 0..=window_size {
                                let rank = ALL_RANKS
                                    [rank_origin + (vertical_dir * (i + j) as i32) as usize];
                                let file =
                                    ALL_FILES[file_origin + (horizontal_dir * i as i32) as usize];
                                let tomb = Square::make_square(rank, file);

                                tomb_squares ^= BitBoard::from_square(tomb);
                                tomb_candidates |= missing_with_target_as_candidate_destiny(
                                    analysis, !color, tomb,
                                );
                            }
                            captured_candidates[tombs.len()] = tomb_candidates;
                            tombs.push(tomb_squares);
                        }
                    }
                }
            }

            // if a tomb cannot be reached by a single candidate, the position is illegal
            for candidates in captured_candidates.iter().take(tombs.len()) {
                if *candidates == EMPTY {
                    analysis.result = Some(Legality::Illegal)
                }
            }

            // before applying the k-groups analysis, we combine the tombs with on-the-board
            // pieces (characterized by their current square location) whose origins are
            // included in the set of candidate missing pieces
            let mut finals = tombs;
            let mut origins_of_finals = captured_candidates;
            for square in *analysis.board.color_combined(!color) {
                // if all the candidate origins of the piece on square are candidate missing,
                // add a new entry to finals
                if analysis.origins(square) & analysis.missing(!color).set_candidates()
                    == analysis.origins(square)
                {
                    origins_of_finals[finals.len()] = analysis.origins(square);
                    finals.push(BitBoard::from_square(square));
                }
            }

            for k in 1..=finals.len() {
                let mut iter = init_iter(finals.len());
                loop {
                    match find_k_group(k, &origins_of_finals, iter) {
                        None => break,
                        Some((group, remaining)) => {
                            if group.popcnt() < k as u32 {
                                analysis.result = Some(Legality::Illegal)
                            }

                            let group_indices = iter & !remaining;
                            iter = remaining;

                            // the destinies of the k-group are now clear
                            let group_destinies =
                                group_indices.fold(EMPTY, |acc, idx| acc | finals[idx.to_index()]);
                            for square in group {
                                progress |= analysis.update_destinies(square, group_destinies)
                            }
                        }
                    }
                }
            }
        }

        progress
    }
}

/// A `BitBoard` encoding the starting square of all the missing pieces of the
/// given color whose destiny may have been the given square.
fn missing_with_target_as_candidate_destiny(
    analysis: &Analysis,
    color: Color,
    target: Square,
) -> BitBoard {
    let mut candidates = EMPTY;
    // TODO: we could be more precise and treat the certainly missing differently
    // from the candidate missing
    for origin in analysis.missing(color).all() {
        if BitBoard::from_square(target) & analysis.destinies(origin) != EMPTY {
            candidates |= BitBoard::from_square(origin)
        }
    }
    candidates
}

/// A `BitBoard` including all the squares in the range 0..n.
fn init_iter(n: usize) -> BitBoard {
    let mut iter = EMPTY;
    for i in 0..n {
        iter |= BitBoard(1 << i as u64);
    }
    iter
}
