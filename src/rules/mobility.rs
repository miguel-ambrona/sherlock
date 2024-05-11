//! Mobility rule.
//!
//! Based on the mobility graph, updates the information about:
//!  - reachable_from_origin

use chess::{Board, Color, Rank, Square, ALL_FILES};

use super::{Analysis, Rule};

#[derive(Debug)]
pub struct MobilityRule {
    mobility_counter: usize,
}

impl Rule for MobilityRule {
    fn new() -> Self {
        MobilityRule {
            mobility_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.mobility_counter = analysis.mobility.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.mobility_counter != analysis.mobility.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        // update reachable_from_origin
        for (color, rank) in [(Color::White, Rank::First), (Color::Black, Rank::Eighth)] {
            for file in ALL_FILES {
                let square = Square::make_square(rank, file);
                let piece = Board::default().piece_on(square).unwrap();
                let reachable = analysis.mobility.value[color.to_index()][piece.to_index()]
                    .reachable_from_source(square);
                progress |= analysis.update_reachable_from_origin(color, file, reachable)
            }
        }
        progress
    }
}
