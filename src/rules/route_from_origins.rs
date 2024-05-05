//! Route from origins rule.
//!
//! This rule makes sure that for every piece on the board there exists a path
//! from its candidate origins to its current square. The candidate origins that
//! do not satisfy this condition are filtered out.

use chess::{BitBoard, EMPTY};

use super::Rule;
use crate::{analysis::Analysis, utils::distance_from_origin};

#[derive(Debug)]
pub struct RouteFromOriginsRule {
    mobility_counter: usize,
    captures_bounds_counter: usize,
    steady_counter: usize,
}

impl Rule for RouteFromOriginsRule {
    fn new() -> Self {
        Self {
            mobility_counter: 0,
            captures_bounds_counter: 0,
            steady_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.mobility_counter = analysis.mobility.counter();
        self.captures_bounds_counter = analysis.captures_bounds.counter();
        self.steady_counter = analysis.steady.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.mobility_counter != analysis.mobility.counter()
            || self.captures_bounds_counter != analysis.captures_bounds.counter()
            || self.steady_counter != analysis.steady.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for square in analysis.board.combined() & !analysis.steady.value {
            let piece = analysis.piece_type_on(square);
            let color = analysis.piece_color_on(square);
            let mut plausible_origins = EMPTY;
            for origin in analysis.origins(square) {
                if square == origin {
                    plausible_origins |= BitBoard::from_square(origin);
                    continue;
                }
                let nb_allowed_captures = analysis.nb_captures_upper_bound(origin);
                if let Some(n) =
                    distance_from_origin(&analysis.mobility.value, origin, square, piece, color)
                {
                    if n <= nb_allowed_captures as u32 {
                        plausible_origins |= BitBoard::from_square(origin);
                    }
                }
            }
            progress |= analysis.update_origins(square, plausible_origins);
        }
        progress
    }
}
