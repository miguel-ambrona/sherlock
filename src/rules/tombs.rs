//! Tombs rule.
//!
//! For every pawn, we compute the squares where it must have captured an enemy
//! piece in its route to its destinies.
//! If a capturing square is common to all its possible destinies, we add this
//! information to their set of tombs.

use std::cmp::min;

use chess::{get_rank, BitBoard, Color, Piece, Rank, Square, EMPTY};

use super::Rule;
use crate::{analysis::Analysis, utils::prom_index};

#[derive(Debug)]
pub struct TombsRule {
    pawn_capture_distances_counter: usize,
    pawn_forced_captures_counter: usize,
    reachable_from_promotion_counter: usize,
    destinies_counter: usize,
    origins_counter: usize,
    captures_bounds_counter: usize,
}

impl Rule for TombsRule {
    fn new() -> Self {
        Self {
            pawn_capture_distances_counter: 0,
            pawn_forced_captures_counter: 0,
            reachable_from_promotion_counter: 0,
            destinies_counter: 0,
            origins_counter: 0,
            captures_bounds_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.pawn_capture_distances_counter = analysis.pawn_capture_distances.counter();
        self.pawn_forced_captures_counter = analysis.pawn_forced_captures.counter();
        self.reachable_from_promotion_counter = analysis.reachable_from_promotion.counter();
        self.destinies_counter = analysis.destinies.counter();
        self.origins_counter = analysis.origins.counter();
        self.captures_bounds_counter = analysis.captures_bounds.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.pawn_capture_distances_counter != analysis.pawn_capture_distances.counter()
            || self.pawn_forced_captures_counter != analysis.pawn_forced_captures.counter()
            || self.reachable_from_promotion_counter != analysis.reachable_from_promotion.counter()
            || self.destinies_counter != analysis.destinies.counter()
            || self.origins_counter != analysis.origins.counter()
            || self.captures_bounds_counter != analysis.captures_bounds.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for origin in (get_rank(Rank::Second) | get_rank(Rank::Seventh)) & !analysis.steady.value {
            let mut tombs = !EMPTY;
            let mut min_distance = 16;

            for destiny in analysis.destinies(origin) {
                // TODO: This condition could be more general and instead be : !missing(origin)
                // Change get_tombs doc example to a Queen on C3 instead of a Bishop when
                // "missing" is supported.
                let final_piece = if analysis.origins(destiny) == BitBoard::from_square(origin) {
                    analysis.board.piece_on(destiny)
                } else {
                    None
                };
                let nb_allowed_captures = analysis.nb_captures_upper_bound(origin) as u32;
                let (tombs_to_destiny, distance_to_destiny) =
                    tombs_to_target(analysis, origin, destiny, nb_allowed_captures, final_piece);
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

/// The squares where the pawn that started the game on `origin` must have
/// captured enemy pieces in order to go from `origin` to `target`, with at most
/// `nb_allowed_captures` captures, according to the current information about
/// the position.
/// If `final_piece` is set, the piece that lands on `target` must
/// be of this type, and a promotion may need to take place.
/// If `final_piece = None`, a promotion may or may not have happened before
/// reaching `target`.
///
/// This function also returns the minimum number of captures necessary to
/// perform the journey as a second argument.
///
/// If the specified route is impossible, this function returns `EMPTY`.
pub fn tombs_to_target(
    analysis: &Analysis,
    origin: Square,
    target: Square,
    nb_allowed_captures: u32,
    final_piece: Option<Piece>,
) -> (BitBoard, u8) {
    let color = match origin.get_rank() {
        Rank::Second => Color::White,
        Rank::Seventh => Color::Black,
        // we only know how to derive non-trivial tombs information for pawns
        _ => return (EMPTY, 0),
    };
    let mut tombs = !EMPTY;
    let mut min_distance = 16;

    // the pawn goes directly to target
    if final_piece.is_none() || final_piece == Some(Piece::Pawn) {
        let distance = analysis.pawn_capture_distances.value[color.to_index()]
            [origin.get_file().to_index()][target.to_index()];
        if distance <= nb_allowed_captures as u8 {
            let path_tombs = analysis.pawn_forced_captures.value[color.to_index()]
                [origin.get_file().to_index()][target.to_index()];
            tombs &= path_tombs;
            min_distance = min(distance, min_distance);
        }
    }

    // the pawn promotes before going to target
    if final_piece != Some(Piece::Pawn) {
        let candidate_promotion_pieces = match final_piece {
            // knights first, they are more likely to be able to reach any square after promotion
            None => vec![Piece::Knight, Piece::Queen, Piece::Rook, Piece::Bishop],
            Some(piece) => vec![piece],
        };
        for promoting_square in get_rank(color.to_their_backrank()) & !analysis.steady.value {
            if tombs == EMPTY {
                break;
            }
            let d1 = analysis.pawn_capture_distances.value[color.to_index()]
                [origin.get_file().to_index()][promoting_square.to_index()];
            if d1 > nb_allowed_captures as u8 {
                continue;
            }
            let path_tombs = analysis.pawn_forced_captures.value[color.to_index()]
                [origin.get_file().to_index()][promoting_square.to_index()];
            for piece in candidate_promotion_pieces.clone() {
                if BitBoard::from_square(target)
                    & analysis.reachable_from_promotion.value[color.to_index()][prom_index(piece)]
                        [promoting_square.get_file().to_index()]
                    == EMPTY
                {
                    continue;
                }
                tombs &= path_tombs;
                min_distance = min(d1, min_distance);
                // the promotion piece is unimportant, we can stop now that a path was found
                break;
            }
        }
    }

    // if at this point tombs == !EMPTY, all routes were impossible, so return EMPTY
    if tombs == !EMPTY {
        return (EMPTY, min_distance);
    }

    (tombs, min_distance)
}
