//! # Sherlock
//!
//! A chess library oriented to creating and solving chess compositions with
//! especial emphasis on retrograde analysis.
//!
//! ## Example
//!
//! This checks whether the position below is reachable from the starting chess
//! position via a sequence of legal moves.
//!
//! ```
//! use chess::Board;
//! use sherlock::is_legal;
//!
//! let board = Board::default();
//! assert!(is_legal(&board));
//! ```

#![deny(missing_docs)]

use chess::{BitBoard, Square, EMPTY};
use rules::ALL_ORIGINS;
use utils::origin_color;

mod analysis;
mod legality;
mod retractor;
mod rules;
mod utils;

pub use crate::{analysis::*, legality::*, retractor::*};

impl Analysis {
    /// Tells whether the piece on the given square was classified as steady
    /// (it has never moved and is still on their starting square).
    ///
    /// <details>
    /// <summary>Visualize this example's position</summary>
    ///
    /// ![FEN](https://backscattering.de/web-boardimage/board.svg?fen=2bqkb2/1ppppp2/8/8/8/8/1PPPPPP1/2BQKB2&colors=lichess-blue&arrows=Gd1,Rd8)
    ///
    /// </details>
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use chess::{Board, Square};
    /// use sherlock::analyze;
    ///
    /// let board =
    ///     Board::from_str("2bqkb2/1ppppp2/8/8/8/8/1PPPPPP1/2BQKB2 w - -").expect("Valid Position");
    /// let analysis = analyze(&board);
    ///
    /// // The white queen cannot possibly have moved, unlike the black queen
    /// assert_eq!(analysis.is_steady(Square::D1), true);
    /// assert_eq!(analysis.is_steady(Square::D8), false);
    /// ```
    #[inline]
    pub fn is_steady(&self, square: Square) -> bool {
        BitBoard::from_square(square) & self.steady.value != EMPTY
    }

    /// Tells whether the piece that started the game on the given square is
    /// known to be missing (it was captured during the game).
    #[inline]
    pub fn is_definitely_missing(&self, origin: Square) -> bool {
        self.missing(origin_color(origin)).mem(origin)
    }

    /// Tells whether the piece that started the game on the given square is
    /// known to be still on the board (possibly in promoted form).
    #[inline]
    pub fn is_definitely_on_the_board(&self, origin: Square) -> bool {
        BitBoard::from_square(origin) & self.missing(origin_color(origin)).all() == EMPTY
    }

    /// The candidate origins of the piece that is on the given square in the
    /// analyzed board.
    ///
    /// <details>
    /// <summary>Visualize this example's position</summary>
    ///
    /// ![FEN](https://backscattering.de/web-boardimage/board.svg?fen=r1bqkbnr/p1pppppp/1p6/R7/4N3/8/1PPPP1PP/2BQKB1R&colors=lichess-blue&arrows=Gb1e4,Gg1e4,Ga2e4,Ba1a5&squares=h4)
    ///
    /// </details>
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use chess::{BitBoard, Board, Square, EMPTY};
    /// use sherlock::analyze;
    ///
    /// let board = Board::from_str("r1bqkbnr/p1pppppp/1p6/R7/4N3/8/1PPPP1PP/2BQKB1R w - -")
    ///     .expect("Valid Position");
    /// let analysis = analyze(&board);
    ///
    /// // The piece on E4 (a white knight) may have started the game on B1 or G1, but
    /// // it can also be the A2-pawn promoted (not the F2 pawn, who could not promote)
    /// assert_eq!(
    ///     analysis.origins(Square::E4),
    ///     BitBoard::from_square(Square::B1)
    ///         | BitBoard::from_square(Square::G1)
    ///         | BitBoard::from_square(Square::A2)
    /// );
    ///
    /// // The rook on A5 has definitely started the game on A1 (it cannot be the A2-pawn
    /// // promoted because it cannot have crossed the black pawn wall after promotion)
    /// assert_eq!(
    ///     analysis.origins(Square::A5),
    ///     BitBoard::from_square(Square::A1)
    /// );
    ///
    /// // This function should not be queried on an empty square, but if you insist...
    /// assert_eq!(analysis.origins(Square::H4), !EMPTY);
    /// ```
    #[inline]
    pub fn origins(&self, square: Square) -> BitBoard {
        self.origins.value[square.to_index()]
    }

    /// The candidate destinies of the piece that started on the given square.
    ///
    /// <details>
    /// <summary>Visualize this example's position</summary>
    ///
    /// ![FEN](https://backscattering.de/web-boardimage/board.svg?fen=r2qkb1r/ppp1pppp/8/7n/b2P4/8/PPPPP1PP/RNBQKBNR&colors=lichess-blue&arrows=Gg8h5,Gg8e3,Bd7d4)
    ///
    /// </details>
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use chess::{BitBoard, Board, Square, EMPTY};
    /// use sherlock::analyze;
    ///
    /// let board = Board::from_str("r2qkb1r/ppp1pppp/8/7n/b2P4/8/PPPPP1PP/RNBQKBNR b KQkq -")
    ///     .expect("Valid Position");
    /// let analysis = analyze(&board);
    ///
    /// // The white pawn on D4 must come from F2 (capturing on E3 and D4)
    /// assert_eq!(
    ///     analysis.destinies(Square::F2),
    ///     BitBoard::from_square(Square::D4)
    /// );
    ///
    /// // On E3 it must have captured the missing black knight, so the knight that started on
    /// // G8 was captured on E3 or is standing on H5 (and the other knight was captured on E3)
    /// assert_eq!(
    ///     analysis.destinies(Square::G8),
    ///     BitBoard::from_square(Square::E3) | BitBoard::from_square(Square::H5)
    /// );
    ///
    /// // On the other hand, the pawn that started on D7 must have been captured on D4
    /// assert_eq!(
    ///     analysis.destinies(Square::D7),
    ///     BitBoard::from_square(Square::D4)
    /// );
    /// ```
    #[inline]
    pub fn destinies(&self, square: Square) -> BitBoard {
        self.destinies.value[square.to_index()]
    }

    /// The squares where opponent pieces have certainly been captured by the
    /// piece that started on the given square.
    ///
    /// <details>
    /// <summary>Visualize this example's position</summary>
    ///
    /// ![FEN](https://backscattering.de/web-boardimage/board.svg?fen=r1b1kb1r/pp1ppppp/2p5/8/8/8/PP1PPPPP/RNQQKBNR&colors=lichess-blue&arrows=Gb6c7,Gd6c7,Gc7b8&squares=e4)
    ///
    /// </details>
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use chess::{BitBoard, Board, Square, EMPTY};
    /// use sherlock::{analyze, Error};
    ///
    /// let board = Board::from_str("r1b1kb1r/pp1ppppp/2p5/8/8/8/PP1PPPPP/RNQQKBNR w KQkq -")
    ///     .expect("Valid Position");
    /// let analysis = analyze(&board);
    ///
    /// // The C2-pawn has certainly promoted (it is now one of the White queens), in order to do so
    /// // it must have captured on C7 and B8 (not D8, as it would have given check to a king that
    /// // never moved); this pawn captured a third piece, but it is not clear where
    /// assert_eq!(
    ///     analysis.get_captures(Square::C2),
    ///     Ok(BitBoard::from_square(Square::C7) | BitBoard::from_square(Square::B8))
    /// );
    ///
    /// // if we provide a square not in the 1st, 2nd, 7th or 8th rank
    /// assert_eq!(
    ///     analysis.get_captures(Square::E4),
    ///     Err(Error::NotOriginSquare)
    /// );
    /// ```
    #[inline]
    pub fn get_captures(&self, square: Square) -> Result<BitBoard, Error> {
        if BitBoard::from_square(square) & ALL_ORIGINS == EMPTY {
            Err(Error::NotOriginSquare)
        } else {
            Ok(self.captures.value[square.to_index()])
        }
    }
}
