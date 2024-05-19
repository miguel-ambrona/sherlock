//! Missing rule.
//!
//! The starting squares that do not appear in the origins of any piece on the
//! board are definitely the starting squares of missing pieces.

use chess::ALL_COLORS;

use super::{Analysis, Rule, COLOR_ORIGINS};

#[derive(Debug)]
pub struct MissingRule {
    origins_counter: usize,
}

impl Rule for MissingRule {
    fn new() -> Self {
        MissingRule { origins_counter: 0 }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.origins_counter = analysis.origins.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.origins_counter != analysis.origins.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for color in ALL_COLORS {
            let mut origins = COLOR_ORIGINS[color.to_index()];
            progress |= analysis.update_certainly_not_missing(color, !origins);

            for square in *analysis.board.color_combined(color) {
                origins &= !analysis.origins(square);
            }

            progress |= analysis.update_certainly_missing(color, origins);
        }

        progress
    }
}
