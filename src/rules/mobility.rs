//! Mobility rule.
//!
//! Based on the mobility graph, updates the information about:
//!  - reachable_from_origin
//!  - reachable_from_promotion
//!  - pawn_capture_distances
//!  - pawn_forced_captures

use chess::{Board, Piece, Square, ALL_COLORS, ALL_FILES, ALL_SQUARES, PROMOTION_PIECES};

use super::{Analysis, Rule};

#[derive(Debug)]
pub struct MobilityRule {
    mobility_counter: usize,
    pawn_capture_distances_counter: usize,
    captures_bounds_counter: usize,
}

impl Rule for MobilityRule {
    fn new() -> Self {
        MobilityRule {
            mobility_counter: 0,
            pawn_capture_distances_counter: 0,
            captures_bounds_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.mobility_counter = analysis.mobility.counter();
        self.pawn_capture_distances_counter = analysis.pawn_capture_distances.counter();
        self.captures_bounds_counter = analysis.captures_bounds.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.mobility_counter != analysis.mobility.counter()
            || self.pawn_capture_distances_counter != analysis.pawn_capture_distances.counter()
            || self.captures_bounds_counter != analysis.captures_bounds.counter()
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

        // update pawn_capture_distances
        for color in ALL_COLORS {
            let rank = color.to_second_rank();
            for file in ALL_FILES {
                let square = Square::make_square(rank, file);
                let distances = analysis.mobility.value[color.to_index()][Piece::Pawn.to_index()]
                    .distances_from_source(square);
                progress |= analysis.update_pawn_capture_distances(color, file, &distances);
            }
        }

        // update pawn_forced_captures
        for color in ALL_COLORS {
            let rank = color.to_second_rank();
            for file in ALL_FILES {
                let square = Square::make_square(rank, file);
                for target in ALL_SQUARES {
                    let n = analysis.pawn_capture_distances(color, file, target);
                    let nb_allowed_captures = analysis.nb_captures_upper_bound(square) as u8;
                    if n == 0 || n > nb_allowed_captures {
                        continue;
                    }
                    let forced = analysis.mobility.value[color.to_index()][Piece::Pawn.to_index()]
                        .forced_captures(square, target);
                    progress |= analysis.update_pawn_forced_captures(color, file, target, forced);
                }
            }
        }

        progress
    }
}
