//! This example corresponds to a problem by Raymond M. Smullyan, from the book
//! "The Chess Mysteries of Sherlock Holmes".
//! See Chapter "Mystery of the Missing Piece".

use std::str::FromStr;

use chess::{Board, Color::White, Piece::Bishop, Square};
use sherlock::{is_legal, ALL_COLORED_PIECES};

fn main() {
    let board = Board::from_str("2nR3K/pk1Rp1p1/p2p4/P1p5/1Pp5/2PP2P1/4P2P/n7 b - -").unwrap();

    // all the pieces that can be placed on h4 leading to a legal position
    let valid_pieces: Vec<_> = ALL_COLORED_PIECES
        .into_iter()
        .filter(|&(color, piece)| {
            #[allow(deprecated)]
            board
                .set_piece(piece, color, Square::H4)
                .as_ref()
                .map_or(false, is_legal)
        })
        .collect();

    assert_eq!(valid_pieces, vec![(White, Bishop)]);
}
