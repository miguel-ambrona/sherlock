//! Route to reachable rule.
//!
//! This rule filters the set of reachable squares of every piece by removing
//! the squares for which there does not exists a path from its original square.

use chess::{get_rank, BitBoard, Board, Color, Piece, Square, ALL_COLORS, EMPTY};

use super::{Rule, COLOR_ORIGINS};
use crate::analysis::Analysis;

#[derive(Debug)]
pub struct RouteToReachable {
    mobility_counter: usize,
    nb_captures_counter: usize,
    steady_counter: usize,
    pawn_capture_distances_counter: usize,
    reachable_from_origin_counter: usize,
}

impl Rule for RouteToReachable {
    fn new() -> Self {
        Self {
            mobility_counter: 0,
            nb_captures_counter: 0,
            steady_counter: 0,
            pawn_capture_distances_counter: 0,
            reachable_from_origin_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.mobility_counter = analysis.mobility.counter();
        self.nb_captures_counter = analysis.nb_captures.counter();
        self.steady_counter = analysis.steady.counter();
        self.pawn_capture_distances_counter = analysis.pawn_capture_distances.counter();
        self.reachable_from_origin_counter = analysis.reachable_from_origin.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.mobility_counter != analysis.mobility.counter()
            || self.nb_captures_counter != analysis.nb_captures.counter()
            || self.steady_counter != analysis.steady.counter()
            || self.pawn_capture_distances_counter != analysis.pawn_capture_distances.counter()
            || self.reachable_from_origin_counter != analysis.reachable_from_origin.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for color in ALL_COLORS {
            for square in COLOR_ORIGINS[color.to_index()] {
                let piece = Board::default().piece_on(square).unwrap();
                let nb_allowed_captures = analysis.nb_captures_upper_bound(square);
                let mut reachable_targets = BitBoard::from_square(square);
                for target in analysis.reachable(square) & !analysis.steady.value {
                    let n = distance_to_target(analysis, square, target, piece, color);
                    if n <= nb_allowed_captures as u8 {
                        reachable_targets |= BitBoard::from_square(target);
                    }
                }
                progress |= analysis.update_reachable(square, reachable_targets);
            }
        }
        progress
    }
}

/// The minimum number of captures necessary (according to the current
/// information about the position) for the given piece of the given color to go
/// from its starting square `origin` to `target`.
/// If this function returns `n`, at least `n` captures are required (but this
/// does not mean that it is possible with exactly `n` captures).
///
/// If the piece is a pawn, it is allowed to promote in order to reach
/// the target.
pub fn distance_to_target(
    analysis: &Analysis,
    origin: Square,
    target: Square,
    piece: Piece,
    color: Color,
) -> u8 {
    // if the piece is a pawn and can promote, we assume it can then reach the
    // target without further captures
    if piece == Piece::Pawn {
        let mut distance = analysis.pawn_capture_distances(color, origin.get_file(), target);
        for promoting_square in get_rank(color.to_their_backrank()) {
            let d = analysis.pawn_capture_distances(color, origin.get_file(), promoting_square);
            if d < distance {
                distance = d;
            }
        }
        return distance;
    }

    if BitBoard::from_square(target) & analysis.reachable_from_origin(color, origin.get_file())
        != EMPTY
    {
        0
    } else {
        16
    }
}

#[cfg(test)]
mod tests {
    use chess::{Color::*, Piece::*};

    use super::*;
    use crate::{
        rules::{MobilityRule, OriginsRule},
        utils::*,
        Analysis, RetractableBoard,
    };

    #[test]
    fn test_distance_to_target() {
        let mut analysis = Analysis::new(&RetractableBoard::default());
        OriginsRule::new().apply(&mut analysis);
        MobilityRule::new().apply(&mut analysis);

        // a queen should be able to go anywhere without captures
        assert_eq!(distance_to_target(&analysis, A1, H8, Queen, Black), 0);

        // a pawn too if it can promote on their original file
        assert_eq!(distance_to_target(&analysis, A2, C4, Pawn, White), 0);

        // even if we disallow A2 -> A3, it can still go A2 -> A4 in one go
        analysis.remove_incoming_edges(Pawn, White, A3);
        MobilityRule::new().apply(&mut analysis);
        assert_eq!(distance_to_target(&analysis, A2, C4, Pawn, White), 0);

        // but also removing A2 -> A4 will force the pawn to capture at least once
        analysis.remove_incoming_edges(Pawn, White, A4);
        MobilityRule::new().apply(&mut analysis);
        assert_eq!(distance_to_target(&analysis, A2, C4, Pawn, White), 1);

        // finally, if we also disallow promotions on B8, it takes at least 2 captures
        analysis.remove_incoming_edges(Pawn, White, B8);
        MobilityRule::new().apply(&mut analysis);
        assert_eq!(distance_to_target(&analysis, A2, C4, Pawn, White), 2);
    }
}
