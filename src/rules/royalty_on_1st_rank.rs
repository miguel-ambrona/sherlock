//! Royalty on 1st rank rule.
//!
//! If a bunch of royal (king, queen, rook) pieces cannot possibly have left
//! their relative fist rank, then they must be in a specific order, e.g., a
//! queen cannot be on the right of a king. If they do not respect a valid
//! order, the position must be illegal.
//!
//! Note: We must be careful with castlings.

use chess::{get_rank, Color, File, Square, ALL_COLORS, ALL_FILES, EMPTY};

use super::{Analysis, Rule};
use crate::Legality;

#[derive(Debug)]
pub struct RoyaltyOn1stRankRule {
    origins_counter: usize,
    reachable_from_origin_counter: usize,
}

impl Rule for RoyaltyOn1stRankRule {
    fn new() -> Self {
        RoyaltyOn1stRankRule {
            origins_counter: 0,
            reachable_from_origin_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.origins_counter = analysis.origins.counter();
        self.reachable_from_origin_counter = analysis.reachable_from_origin.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.origins_counter != analysis.origins.counter()
            || self.reachable_from_origin_counter != analysis.reachable_from_origin.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        for color in ALL_COLORS {
            let royalty = royalty_on_1st_rank(analysis, color);

            // we could expect the royalty files to be in alphabetical order for
            // legality, however, castling spoils this nice invariant; instead,
            // we will check that the "D" file (if it exists) is in its
            // alphabetical position

            let royalty_indices = royalty.iter().map(|f| f.to_index()).collect::<Vec<_>>();
            let mut sorted_royalty_indices = royalty_indices.clone();
            sorted_royalty_indices.sort();

            // the D file has index 3
            if royalty_indices.iter().position(|&i| i == 3)
                != sorted_royalty_indices.iter().position(|&i| i == 3)
            {
                analysis.result = Some(Legality::Illegal);
            }
        }

        false
    }
}

// Returns the files of the current location of royal pieces (King, Queen, Rook)
// of the given color that cannot have possibly have left their relative first
// rank.
fn royalty_on_1st_rank(analysis: &Analysis, color: Color) -> Vec<File> {
    ALL_FILES
        .into_iter()
        .filter_map(|file| {
            let square = Square::make_square(color.to_my_backrank(), file);
            let origins = analysis.origins(square);
            let origin_file = origins.to_square().get_file();
            if origins.popcnt() == 1
                && analysis.reachable_from_origin(color, origin_file)
                    & !get_rank(color.to_my_backrank())
                    == EMPTY
            {
                Some(origin_file)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use chess::{BitBoard, Rank};

    use super::*;
    use crate::RetractableBoard;

    #[test]
    fn test_royalty_on_1st_rank() {
        use File::*;
        let board =
            RetractableBoard::from_fen("r2qk2r/pppppppp/8/8/8/8/PPPPPPPP/3K4 w - -").unwrap();

        let mut analysis = Analysis::new(&board);

        assert_eq!(royalty_on_1st_rank(&analysis, Color::White), vec![]);

        // Learn that no white piece could have reached their 1st rank
        for file in ALL_FILES {
            analysis.update_reachable_from_origin(Color::White, file, get_rank(Rank::First));
        }

        assert_eq!(royalty_on_1st_rank(&analysis, Color::White), vec![]);

        // Learn the origins of A1, D1, F1, G1 are, resp., A1, E1, D1, H1.
        analysis.update_origins(Square::A1, BitBoard::from_square(Square::A1));
        analysis.update_origins(Square::D1, BitBoard::from_square(Square::E1));
        analysis.update_origins(Square::F1, BitBoard::from_square(Square::D1));
        analysis.update_origins(Square::G1, BitBoard::from_square(Square::H1));

        assert_eq!(
            royalty_on_1st_rank(&analysis, Color::White),
            vec![A, E, D, H]
        );
    }
}
