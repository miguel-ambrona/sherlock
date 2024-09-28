//! Refine origins rule.
//!
//! This rule exploits the fact that if there is a group of k pieces with k
//! combined candidate origins, those origins cannot be origins of any other
//! piece.

use chess::{get_file, get_rank, BitBoard, Piece, ALL_COLORS, EMPTY};

use super::{sum_lower_bounds_nb_captures, Analysis, Rule, COLOR_ORIGINS};
use crate::{utils::find_k_group, Legality::Illegal};

#[derive(Debug)]
pub struct RefineOriginsRule {
    origins_counter: usize,
    nb_captures_counter: usize,
    reachable_from_origin_counter: usize,
    pawn_capture_distances_counter: usize,
}

impl Rule for RefineOriginsRule {
    fn new() -> Self {
        RefineOriginsRule {
            origins_counter: 0,
            nb_captures_counter: 0,
            reachable_from_origin_counter: 0,
            pawn_capture_distances_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.origins_counter = analysis.origins.counter();
        self.nb_captures_counter = analysis.nb_captures.counter();
        self.reachable_from_origin_counter = analysis.reachable_from_origin.counter();
        self.pawn_capture_distances_counter = analysis.pawn_capture_distances.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.origins_counter != analysis.origins.counter()
            || self.nb_captures_counter != analysis.nb_captures.counter()
            || self.reachable_from_origin_counter != analysis.reachable_from_origin.counter()
            || self.pawn_capture_distances_counter != analysis.pawn_capture_distances.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for color in ALL_COLORS {
            // We iterate up to k = 10, since that is the maximum number of candidate
            // origins of any piece after applying the origins rule.
            for k in 1..=10 {
                let mut iter = *analysis.board.color_combined(color);
                loop {
                    match find_k_group(k, &analysis.origins.value, iter) {
                        None => break,
                        Some((group, remaining)) => {
                            let group_indices = iter & !remaining;
                            // we remove the k-group from the origins of the remaining
                            iter = remaining;
                            for square in iter {
                                let square_origins = analysis.origins(square) & !group;
                                progress |= analysis.update_origins(square, square_origins);
                            }

                            // we remove the k-group from the set of candidate missing pieces
                            progress |= analysis.update_certainly_not_missing(color, group);

                            // the destinies of the k-group are limited by the group_indices
                            for origin in group {
                                progress |= analysis.update_destinies(origin, group_indices)
                            }

                            // a simple heuristic to conclude ASAP that pawns did not capture
                            if group_indices & analysis.board.pieces(Piece::Pawn) == group_indices
                                && group_indices.popcnt() > 1
                            {
                                let nb_opponents = analysis.board.color_combined(!color).popcnt();
                                let nb_other_captures = sum_lower_bounds_nb_captures(
                                    analysis,
                                    COLOR_ORIGINS[color.to_index()] & !group,
                                );

                                // the group of (at least 2) pawns captured at most once
                                if nb_opponents + nb_other_captures as u32 >= 15 {
                                    for origin in group {
                                        let destinies = group_indices & get_file(origin.get_file());
                                        if destinies.popcnt() == 1 {
                                            progress |=
                                                analysis.update_destinies(origin, destinies);
                                            progress |= analysis.update_origins(
                                                destinies.to_square(),
                                                BitBoard::from_square(origin),
                                            );
                                        }
                                    }
                                }

                                // if the group has exactly 2 pawns, we will check if one of the
                                // 2 origin-target possibilities is illegal due to an excessive
                                // number of captures.
                                if group.popcnt() == 2 {
                                    let o1 = group.to_square();
                                    let o1_bb = BitBoard::from_square(o1);
                                    let o2_bb = group ^ o1_bb;
                                    let o2 = o2_bb.to_square();

                                    let t1 = group_indices.to_square();
                                    let t1_bb = BitBoard::from_square(t1);
                                    let t2_bb = group_indices ^ t1_bb;
                                    let t2 = t2_bb.to_square();

                                    let mut nb_missing_opp_that_never_left_first_rank = 0;
                                    for missing in analysis.missing(!color).certainly_in_the_set()
                                        & get_rank(color.to_their_backrank())
                                    {
                                        if analysis
                                            .reachable_from_origin(!color, missing.get_file())
                                            & !get_rank(color.to_their_backrank())
                                            == EMPTY
                                        {
                                            nb_missing_opp_that_never_left_first_rank += 1;
                                        }
                                    }

                                    let bound_option1 = nb_opponents as u8
                                        + analysis.pawn_capture_distances(color, o1.get_file(), t1)
                                        + analysis.pawn_capture_distances(color, o2.get_file(), t2)
                                        + nb_other_captures as u8
                                        + nb_missing_opp_that_never_left_first_rank;

                                    let bound_option2 = nb_opponents as u8
                                        + analysis.pawn_capture_distances(color, o1.get_file(), t2)
                                        + analysis.pawn_capture_distances(color, o2.get_file(), t1)
                                        + nb_other_captures as u8
                                        + nb_missing_opp_that_never_left_first_rank;

                                    if bound_option1 > 16 && bound_option2 > 16 {
                                        analysis.result = Some(Illegal);
                                        return true;
                                    }

                                    if bound_option1 > 16 {
                                        progress |= analysis.update_destinies(o1, t2_bb);
                                        progress |= analysis.update_destinies(o2, t1_bb);
                                        progress |= analysis.update_origins(t2, o1_bb);
                                        progress |= analysis.update_origins(t1, o2_bb);
                                    } else if bound_option2 > 16 {
                                        progress |= analysis.update_destinies(o1, t1_bb);
                                        progress |= analysis.update_destinies(o2, t2_bb);
                                        progress |= analysis.update_origins(t1, o1_bb);
                                        progress |= analysis.update_origins(t2, o2_bb);
                                    }
                                }
                            } // end of pawn heuristic
                        }
                    }
                }
            }
        }
        progress
    }
}

#[cfg(test)]
mod tests {

    use chess::EMPTY;

    use super::*;
    use crate::{utils::*, RetractableBoard};

    #[test]
    fn test_refine_origins_rule() {
        let mut analysis = Analysis::new(&RetractableBoard::default());
        let destinies_rule = RefineOriginsRule::new();

        destinies_rule.apply(&mut analysis);

        // we should not have any information on destinies yet
        assert_eq!(analysis.destinies(E1), !EMPTY);
        assert_eq!(analysis.destinies(E7), !EMPTY);

        // learn that E1 is the only candidate origin of the piece on A1
        analysis.update_origins(A1, bitboard_of_squares(&[E1]));
        destinies_rule.apply(&mut analysis);

        // the destinies of E1 must have been updated to A1
        assert_eq!(analysis.destinies(E1), bitboard_of_squares(&[A1]));

        // others are still uncertain
        assert_eq!(analysis.destinies(E7), !EMPTY);
    }
}
