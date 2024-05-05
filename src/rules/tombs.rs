//! Tombs rule.
//!
//! For every pawn, we compute the squares where it must have captured an enemy
//! piece in its route to its destinies.
//! If a capturing square is common to all its possible destinies, we add this
//! information to their set of tombs.

use std::cmp::min;

use chess::{get_rank, BitBoard, Rank, EMPTY};

use super::Rule;
use crate::{analysis::Analysis, utils::tombs_to_target};

#[derive(Debug)]
pub struct TombsRule {
    mobility_counter: usize,
    destinies_counter: usize,
    captures_bounds_counter: usize,
}

impl Rule for TombsRule {
    fn new() -> Self {
        Self {
            mobility_counter: 0,
            destinies_counter: 0,
            captures_bounds_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.mobility_counter = analysis.mobility.counter();
        self.destinies_counter = analysis.destinies.counter();
        self.captures_bounds_counter = analysis.captures_bounds.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.mobility_counter != analysis.mobility.counter()
            || self.destinies_counter != analysis.destinies.counter()
            || self.captures_bounds_counter != analysis.captures_bounds.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for origin in (get_rank(Rank::Second) | get_rank(Rank::Seventh)) & !analysis.steady.value {
            let mut tombs = !EMPTY;
            let mut min_distance = 16;

            for destiny in analysis.destinies(origin) {
                let final_piece = if analysis.origins(destiny) == BitBoard::from_square(origin) {
                    analysis.board.piece_on(destiny)
                } else {
                    None
                };
                let nb_allowed_captures = analysis.nb_captures_upper_bound(origin) as u32;
                let (tombs_to_destiny, distance_to_destiny) = tombs_to_target(
                    &analysis.mobility.value,
                    origin,
                    destiny,
                    nb_allowed_captures,
                    final_piece,
                );
                tombs &= tombs_to_destiny;
                min_distance = min(distance_to_destiny, min_distance);
            }
            if tombs != !EMPTY {
                progress |= analysis.update_tombs(origin, tombs);
                progress |= analysis.update_captures_lower_bound(origin, min_distance as i32);
            }
        }

        progress
    }
}
