//! Mobility rule.
//!
//! Based on the mobility graph, updates the information about:
//!  - reachable_from_origin
//!  - reachable_from_promotion

use chess::{Board, Square, ALL_COLORS, ALL_FILES, PROMOTION_PIECES};

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
        for color in ALL_COLORS {
            let rank = color.to_my_backrank();
            for file in ALL_FILES {
                let square = Square::make_square(rank, file);
                let piece = Board::default().piece_on(square).unwrap();
                let reachable = analysis.mobility.value[color.to_index()][piece.to_index()]
                    .reachable_from_source(square);
                progress |= analysis.update_reachable_from_origin(color, file, reachable)
            }
        }

        // update reachable_from_promotion
        for color in ALL_COLORS {
            let rank = color.to_their_backrank();
            for piece in PROMOTION_PIECES {
                for file in ALL_FILES {
                    let square = Square::make_square(rank, file);
                    let reachable = analysis.mobility.value[color.to_index()][piece.to_index()]
                        .reachable_from_source(square);
                    progress |=
                        analysis.update_reachable_from_promotion(color, piece, file, reachable)
                }
            }
        }

        progress
    }
}
