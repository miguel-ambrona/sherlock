//! File counting rule.
//!
//! We say a file is closed it it contains opposing enemy pawns.
//! Every officer capture can "open" at most 1 file, whereas every
//! pawn capture can "open" at most 2 files.
//! This rule exploits this fact.

use chess::{get_file, BitBoard, Color, File, Piece, ALL_FILES, EMPTY};

use super::Rule;
use crate::{analysis::Analysis, Legality::Illegal, RetractableBoard};

#[derive(Debug)]
pub struct FileCountingRule {
    applied: bool,
}

impl Rule for FileCountingRule {
    fn new() -> Self {
        FileCountingRule { applied: false }
    }

    fn update(&mut self, _analysis: &Analysis) {
        self.applied = true;
    }

    fn is_applicable(&self, _analysis: &Analysis) -> bool {
        !self.applied
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let pawns = analysis.board.pieces(Piece::Pawn);
        let officers = analysis.board.combined() & !pawns;
        let max_nb_non_closed_files = 2 * (16 - pawns.popcnt()) + (16 - officers.popcnt());

        if 8 - closed_files(&analysis.board).len() > max_nb_non_closed_files as usize {
            analysis.result = Some(Illegal);
        }

        false
    }
}

/// Returns the files where there exist opposing enemy pawns.
fn closed_files(board: &RetractableBoard) -> Vec<File> {
    let pawns = board.pieces(Piece::Pawn);
    let white_pawns = pawns & board.color_combined(Color::White);
    let black_pawns = pawns & board.color_combined(Color::Black);

    let white_pawns_with_opposition = (1..8).fold(EMPTY, |acc, i| {
        acc | BitBoard(white_pawns.0 & black_pawns.0 >> (8 * i))
    });

    ALL_FILES
        .into_iter()
        .filter(|file| get_file(*file) & white_pawns_with_opposition != EMPTY)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closed_files() {
        [
            (
                "4k3/2p5/PPp1Pp2/1P3P2/pp1p1P2/P2p4/3P4/4K3 w - -",
                vec![File::A, File::D, File::F],
            ),
            (
                "4k3/3p4/PP2Pp1P/2P3P1/pp2p3/2p2Ppp/3P4/4K3 w - -",
                vec![File::D, File::F],
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -",
                ALL_FILES.to_vec(),
            ),
        ]
        .iter()
        .for_each(|(fen, expected)| {
            let board = RetractableBoard::from_fen(fen).expect("Valid Position");
            assert_eq!(closed_files(&board), *expected);
        })
    }
}
