//! Tombs rule.
//!
//! We make sure that all known capturing squares can be reached by an opponent
//! piece to be captured. This allows us to deduce new information about e.g.
//! the destinies of a pieces.

use chess::{BitBoard, Color, Square, ALL_COLORS, EMPTY};

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
                    tombs.push(tomb);
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
                    finals.push(square);
                }
            }

            for k in 1..finals.len() {
                let mut iter = init_iter(finals.len());
                loop {
                    match find_k_group(k, &origins_of_finals, iter) {
                        None => break,
                        Some((group, remaining)) => {
                            let group_indices = iter & !remaining;
                            iter = remaining;

                            // the destinies of the k-group are now clear
                            let group_destinies = group_indices.fold(EMPTY, |acc, idx| {
                                acc | BitBoard::from_square(finals[idx.to_index()])
                            });
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
