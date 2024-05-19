//! Refine origins rule.
//!
//! This rule exploits the fact that if there is a group of k pieces with k
//! combined candidate origins, those origins cannot be origins of any other
//! piece.

use chess::ALL_COLORS;

use super::{Analysis, Rule};
use crate::utils::find_k_group;

#[derive(Debug)]
pub struct RefineOriginsRule {
    origins_counter: usize,
}

impl Rule for RefineOriginsRule {
    fn new() -> Self {
        RefineOriginsRule { origins_counter: 0 }
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

    use chess::{Board, EMPTY};

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_refine_origins_rule() {
        let mut analysis = Analysis::new(&Board::default());
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
