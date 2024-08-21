//! Surpassed pawns rule.
//!
//! If a white pawn is on the same file as a black pawn, but a higher rank, and
//! they are known to both originally come from that file, we know that
//! (together) they must have captured at least twice.
//!
//! Having this information into account, we may deduce that the necessary
//! number of captures to reach a position exceeds the total number of captures
//! that have taken place.

use std::cmp::max;

use chess::{get_file, Color, File, Piece, Rank, Square, ALL_FILES};

use super::{Analysis, Rule, ALL_ORIGINS};
use crate::Legality;

#[derive(Debug)]
pub struct SurpassedPawnsRule {
    nb_captures_counter: usize,
    origins_counter: usize,
}

impl Rule for SurpassedPawnsRule {
    fn new() -> Self {
        SurpassedPawnsRule {
            nb_captures_counter: 0,
            origins_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.nb_captures_counter = analysis.nb_captures.counter();
        self.origins_counter = analysis.origins.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.nb_captures_counter != analysis.nb_captures.counter()
            || self.origins_counter != analysis.origins.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut min_nb_captures: i32 = ALL_ORIGINS
            .map(|origin| analysis.nb_captures_lower_bound(origin))
            .sum();

        for file in surpassed_pawns_files(analysis) {
            let white_origin = Square::make_square(Rank::Second, file);
            let black_origin = Square::make_square(Rank::Seventh, file);
            let nb_captures_together = analysis.nb_captures_lower_bound(white_origin)
                + analysis.nb_captures_lower_bound(black_origin);
            min_nb_captures += max(0, 2 - nb_captures_together);
        }

        if min_nb_captures as u32 + analysis.board.combined().popcnt() > 32 {
            analysis.result = Some(Legality::Illegal);
        }

        false
    }
}

/// Returns `Some(r)` if there exists a pawn of the given color on the given
/// file that is known to have started the game on that file, where `r` is its
/// current rank. Returns `None` otherwise.
fn rank_of_file_pawn(analysis: &Analysis, file: File, color: Color) -> Option<Rank> {
    for square in
        get_file(file) & analysis.board.pieces(Piece::Pawn) & analysis.board.color_combined(color)
    {
        let origins = analysis.origins(square);
        if origins.popcnt() == 1 && origins.to_square().get_file() == file {
            return Some(square.get_rank());
        }
    }
    None
}

/// Returns a vector of files where there are surpassed pawns in the given
/// board, i.e. files where there exist a white pawn and a black pawn, known to
/// originally come from that file and such that the black pawn is in a lower
/// rank than the white one.
fn surpassed_pawns_files(analysis: &Analysis) -> Vec<File> {
    ALL_FILES
        .into_iter()
        .filter(|file| {
            if let Some(white_rank) = rank_of_file_pawn(analysis, *file, Color::White) {
                if let Some(black_rank) = rank_of_file_pawn(analysis, *file, Color::Black) {
                    if black_rank < white_rank {
                        return true;
                    }
                }
            }
            false
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use chess::{BitBoard, Board, EMPTY};

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_surpassed_pawns_files() {
        let board = Board::from_str("4k3/7P/1P1p2P1/1p4p1/3P2P1/1P6/7p/4K3 w - -").unwrap();
        let mut analysis = Analysis::new(&board.into());

        assert_eq!(surpassed_pawns_files(&analysis), vec![]);

        // learn that the B6 and B5 pawns come from the B file
        analysis.update_origins(B6, BitBoard::from_square(B2));
        analysis.update_origins(B5, BitBoard::from_square(B7));

        assert_eq!(surpassed_pawns_files(&analysis), vec![File::B]);

        // learn that the D4 and D6 pawns come from the D file
        analysis.update_origins(D4, BitBoard::from_square(D2));
        analysis.update_origins(D6, BitBoard::from_square(D7));

        assert_eq!(surpassed_pawns_files(&analysis), vec![File::B]);

        // learn that the G4 and G5 pawns come from the G file
        analysis.update_origins(G4, BitBoard::from_square(G2));
        analysis.update_origins(G5, BitBoard::from_square(G7));

        assert_eq!(surpassed_pawns_files(&analysis), vec![File::B]);

        // take it back, it was the G6 pawn the one that started on G
        analysis.update_origins(G4, EMPTY);
        analysis.update_origins(G6, BitBoard::from_square(G2));

        assert_eq!(surpassed_pawns_files(&analysis), vec![File::B, File::G]);

        // learn that the H7 pawn comes from H2
        analysis.update_origins(H7, BitBoard::from_square(H2));

        assert_eq!(surpassed_pawns_files(&analysis), vec![File::B, File::G]);

        // and that the H2 pawn comes from H7
        analysis.update_origins(H2, BitBoard::from_square(H7));

        assert_eq!(
            surpassed_pawns_files(&analysis),
            vec![File::B, File::G, File::H]
        );
    }
}
