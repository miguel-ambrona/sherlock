//! This example includes a problem by Raymond M. Smullyan, from the book
//! "The Chess Mysteries of Sherlock Holmes". See Chapter "Two Bagatelles".

use std::str::FromStr;

use chess::{Board, CastleRights, Color::Black, ALL_CASTLE_RIGHTS};
use sherlock::is_legal;

fn main() {
    let board = Board::from_str("r1b1k2r/p1p1p1pp/1p3p2/8/8/P7/1PPPPPPP/2BQKB2 b - -").unwrap();

    // all the castle rights (for Black) under which the position is legal
    let valid_castling_rights: Vec<_> = ALL_CASTLE_RIGHTS
        .into_iter()
        .filter(|&castling_rights| {
            let mut board_copy = board;
            #[allow(deprecated)]
            board_copy.add_castle_rights(Black, castling_rights);
            is_legal(&board_copy)
        })
        .collect();

    assert_eq!(
        valid_castling_rights,
        vec![CastleRights::NoRights, CastleRights::QueenSide]
    );
}
