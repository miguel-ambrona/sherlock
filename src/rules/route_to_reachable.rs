//! Route to reachable rule.
//!
//! This rule filters the set of reachable squares of every piece by removing
//! the squares for which there does not exists a path from its original square.

use chess::{BitBoard, Board, ALL_COLORS};

use super::{Rule, COLOR_ORIGINS};
use crate::{analysis::Analysis, utils::distance_to_target};

#[derive(Debug)]
pub struct RouteToReachable {
    mobility_counter: usize,
    captures_bounds_counter: usize,
    steady_counter: usize,
}

impl Rule for RouteToReachable {
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

        for color in ALL_COLORS {
            for square in COLOR_ORIGINS[color.to_index()] {
                let piece = Board::default().piece_on(square).unwrap();
                let nb_allowed_captures = analysis.nb_captures_upper_bound(square);
                let mut reachable_targets = BitBoard::from_square(square);
                for target in analysis.reachable(square) & !analysis.steady.value {
                    if let Some(n) =
                        distance_to_target(&analysis.mobility.value, square, target, piece, color)
                    {
                        if n <= nb_allowed_captures as u32 {
                            reachable_targets |= BitBoard::from_square(target);
                        }
                    }
                }
                progress |= analysis.update_reachable(square, reachable_targets);
            }
        }
        progress
    }
}
