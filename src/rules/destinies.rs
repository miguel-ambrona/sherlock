//! Destinies rule.
//!
//! If the piece currently on `s` has only one candidate origin `o`, then the
//! destiny of `o` must be `s`.

use chess::{BitBoard, Board};

use super::{Rule, State};

#[derive(Debug)]
pub struct DestiniesRule {
    origins_counter: usize,
}

impl Rule for DestiniesRule {
    fn new(_board: &Board) -> Self {
        DestiniesRule { origins_counter: 0 }
    }

    fn is_applicable(&self, state: &State) -> bool {
        self.origins_counter != state.origins.counter() || self.origins_counter == 0
    }

    fn apply(&mut self, state: &mut State) {
        let mut progress = false;

        for square in *state.board.combined() & !state.steady.value {
            if state.origins(square).popcnt() == 1 {
                let origin = state.origins(square).to_square();
                progress |= state.update_destinies(origin, BitBoard::from_square(square))
            }
        }

        // update the rule state
        self.origins_counter = state.origins.counter();

        // report any progress
        state.destinies.increase_counter(progress);
        state.progress |= progress;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chess::{Board, EMPTY};

    use super::*;
    use crate::{rules::Rule, state::State, utils::*};

    #[test]
    fn test_destinies_rule() {
        let board = Board::from_str("1k6/8/8/8/8/8/8/K7 w - -").expect("Valid Position");
        let mut state = State::new(&board);
        let mut destinies_rule = DestiniesRule::new(&board);

        destinies_rule.apply(&mut state);

        // we should not have any information on destinies yet
        assert_eq!(state.destinies(E1), !EMPTY);
        assert_eq!(state.destinies(E7), !EMPTY);

        // learn that E1 is the only candidate origin of the piece on A1
        state.update_origins(A1, bitboard_of_squares(&[E1]));
        destinies_rule.apply(&mut state);

        // the destinies of E1 must have been updated to A1
        assert_eq!(state.destinies(E1), bitboard_of_squares(&[A1]));

        // others are still uncertain
        assert_eq!(state.destinies(E7), !EMPTY);
    }
}
