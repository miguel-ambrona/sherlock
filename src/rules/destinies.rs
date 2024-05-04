//! Destinies rule.
//!
//! If the piece currently on `s` has only one candidate origin `o`, then the
//! destiny of `o` must be `s`.

use chess::BitBoard;

use super::{Analysis, Rule};

#[derive(Debug)]
pub struct DestiniesRule {
    origins_counter: usize,
    reachable_counter: usize,
}

impl Rule for DestiniesRule {
    fn new() -> Self {
        DestiniesRule {
            origins_counter: 0,
            reachable_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.origins_counter = analysis.origins.counter();
        self.reachable_counter = analysis.reachable.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.origins_counter != analysis.origins.counter()
            || self.reachable_counter != analysis.reachable.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        for square in *analysis.board.combined() {
            if analysis.origins(square).popcnt() == 1 {
                let origin = analysis.origins(square).to_square();
                progress |= analysis.update_destinies(origin, BitBoard::from_square(square))
            }

            let reachable_destinies = analysis.destinies(square) & analysis.reachable(square);
            progress |= analysis.update_destinies(square, reachable_destinies)
        }
        progress
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chess::{Board, EMPTY};

    use super::*;
    use crate::{rules::Rule, utils::*};

    #[test]
    fn test_destinies_rule() {
        let board = Board::from_str("1k6/8/8/8/8/8/8/K7 w - -").expect("Valid Position");
        let mut analysis = Analysis::new(&board);
        let destinies_rule = DestiniesRule::new();

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
