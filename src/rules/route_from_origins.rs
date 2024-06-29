//! Route from origins rule.
//!
//! This rule makes sure that for every piece on the board there exists a path
//! from its candidate origins to its current square. The candidate origins that
//! do not satisfy this condition are filtered out.

use chess::{get_rank, BitBoard, Color, Piece, Square, EMPTY};

use super::Rule;
use crate::analysis::Analysis;

#[derive(Debug)]
pub struct RouteFromOriginsRule {
    pawn_capture_distances_counter: usize,
    pawn_forced_captures_counter: usize,
    reachable_from_promotion_counter: usize,
    nb_captures_counter: usize,
    steady_counter: usize,
}

impl Rule for RouteFromOriginsRule {
    fn new() -> Self {
        Self {
            pawn_capture_distances_counter: 0,
            pawn_forced_captures_counter: 0,
            reachable_from_promotion_counter: 0,
            nb_captures_counter: 0,
            steady_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.pawn_capture_distances_counter = analysis.pawn_capture_distances.counter();
        self.pawn_forced_captures_counter = analysis.pawn_forced_captures.counter();
        self.reachable_from_promotion_counter = analysis.reachable_from_promotion.counter();
        self.nb_captures_counter = analysis.nb_captures.counter();
        self.steady_counter = analysis.steady.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.pawn_capture_distances_counter != analysis.pawn_capture_distances.counter()
            || self.pawn_forced_captures_counter != analysis.pawn_forced_captures.counter()
            || self.reachable_from_promotion_counter != analysis.reachable_from_promotion.counter()
            || self.nb_captures_counter != analysis.nb_captures.counter()
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
                let n = distance_from_origin(analysis, origin, square, piece, color);
                if n <= nb_allowed_captures as u8 {
                    plausible_origins |= BitBoard::from_square(origin);
                }
            }
            progress |= analysis.update_origins(square, plausible_origins);
        }
        progress
    }
}

/// The minimum number of captures necessary (according to the current
/// information about the position) for the piece of the given color to go from
/// its starting square `origin`  to `target`, and end up being the given
/// `piece` type.
/// If this function returns `n`, at least `n` captures are required (but this
/// does not mean that it is possible with exactly `n` captures).
/// We return `16` when the route is impossible.
///
/// Note that if the origin square is in the (relative) 2nd rank, the pawn may
/// have to promote before becoming the desired piece.
pub fn distance_from_origin(
    analysis: &Analysis,
    origin: Square,
    target: Square,
    piece: Piece,
    color: Color,
) -> u8 {
    if piece == Piece::Pawn {
        analysis.pawn_capture_distances(color, origin.get_file(), target)
    } else if (BitBoard::from_square(origin) & get_rank(color.to_my_backrank())) != EMPTY {
        if BitBoard::from_square(target) & analysis.reachable_from_origin(color, origin.get_file())
            != EMPTY
        {
            0
        } else {
            16
        }
    } else {
        // the distance after promoting
        let mut distance = 16;
        for promoting_square in get_rank(color.to_their_backrank()) {
            let distance_to_promotion =
                analysis.pawn_capture_distances(color, origin.get_file(), promoting_square);
            if distance_to_promotion >= distance {
                continue;
            }
            if BitBoard::from_square(target)
                & analysis.reachable_from_promotion(color, piece, promoting_square.get_file())
                != EMPTY
            {
                distance = distance_to_promotion;
            }
        }
        distance
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
    fn test_distance_from_origin() {
        let mut analysis = Analysis::new(&RetractableBoard::default());
        OriginsRule::new().apply(&mut analysis);
        MobilityRule::new().apply(&mut analysis);

        // a bishop on H5 cannot have come from C1, a dark square
        assert_eq!(distance_from_origin(&analysis, C1, H5, Bishop, White), 16);

        // but it may have come from F1, a light square, no captures needed
        assert_eq!(distance_from_origin(&analysis, B1, H5, Bishop, White), 0);

        // it can also have come from B2, although it is a dark square, because
        // it could have been a promoted pawn, at least a capture is needed though,
        // to switch to a file with a light promoting square
        assert_eq!(distance_from_origin(&analysis, B2, H5, Bishop, White), 1);

        // or from B7 if the bishop were Black (as B1 is light)
        assert_eq!(distance_from_origin(&analysis, B7, H5, Bishop, Black), 0);

        // let us remove some graph connections
        analysis.remove_outgoing_edges(Bishop, White, A8);
        analysis.remove_outgoing_edges(Bishop, White, C8);
        MobilityRule::new().apply(&mut analysis);

        // now we cannot promote on A8 or C8, it has to be E8 which takes 3 captures
        assert_eq!(distance_from_origin(&analysis, B2, H5, Bishop, White), 3);

        // a black pawn on C3 can come from F7, but it takes 3 captures
        assert_eq!(distance_from_origin(&analysis, F7, C3, Pawn, Black), 3);

        // but it cannot come from H7, because it would not be a pawn after a promotion
        assert_eq!(distance_from_origin(&analysis, H7, C3, Pawn, Black), 16);

        // if we remove the connection E6 -> D5, it can still come from F7
        analysis.remove_incoming_edges(Pawn, Black, D5);
        MobilityRule::new().apply(&mut analysis);
        assert_eq!(distance_from_origin(&analysis, F7, C3, Pawn, Black), 3);

        // but also removing E5 -> D4 will disconnect it from F7
        analysis.remove_incoming_edges(Pawn, Black, D4);
        MobilityRule::new().apply(&mut analysis);
        assert_eq!(distance_from_origin(&analysis, F7, C3, Pawn, Black), 16);
    }
}
