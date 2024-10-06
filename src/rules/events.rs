//! Events rule.
//!
//! TODO

use chess::{BitBoard, Color, Piece, Square, EMPTY};

use crate::utils::{E2, F1, F3, G2};

use super::{Analysis, Rule};

#[derive(Debug)]
pub struct EventsRule {
    origins_counter: usize,
    events_counter: usize,
}

impl Rule for EventsRule {
    fn new() -> Self {
        EventsRule {
            origins_counter: 0,
            events_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.origins_counter = analysis.origins.counter();
        self.events_counter = analysis.events.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.origins_counter != analysis.origins.counter()
            || self.events_counter != analysis.events.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut progress = false;

        let pawns = analysis.board.pieces(Piece::Pawn);
        let white_pawns = analysis.board.color_combined(Color::White) & pawns;

        if white_pawns & bitboard_of_squares(&[F3, G2]) == bitboard_of_squares(&[F3, G2]) {
            progress |= analysis.add_event(E2, F1);
        }

        progress
    }
}

pub(crate) fn bitboard_of_squares(squares: &[Square]) -> BitBoard {
    squares
        .iter()
        .fold(EMPTY, |acc, s| acc | BitBoard::from_square(*s))
}
