use std::fmt;

use chess::{Piece, Square};

/// Represent a ChessRetraction in memory.
#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Default, Debug, Hash)]
pub struct ChessRetraction {
    source: Square,
    target: Square,
    uncaptured: Option<Piece>,
    unpromotion: bool,
}

impl ChessRetraction {
    /// Create a new chess retraction, given a source (the square we retract
    /// from), a destination (the square we retract into), an optional piece
    /// being "uncaptured", the retraction kind (normal, unpromotion or
    /// un_en_passant) and information about the castling and en_passant
    /// flags after the retraction.
    #[inline]
    pub fn new(
        source: Square,
        target: Square,
        uncaptured: Option<Piece>,
        unpromotion: bool,
    ) -> ChessRetraction {
        ChessRetraction {
            source,
            target,
            uncaptured,
            unpromotion,
        }
    }

    /// The retraction source square (the square the piece is retracting from).
    #[inline]
    pub fn source(&self) -> Square {
        self.source
    }

    /// The retraction target square (the square the piece is retracting to).
    #[inline]
    pub fn target(&self) -> Square {
        self.target
    }

    /// The retraction uncatured piece type, if any.
    #[inline]
    pub fn uncaptured(&self) -> Option<Piece> {
        self.uncaptured
    }

    /// Is the retraction an unpromotion?
    #[inline]
    pub fn unpromotion(&self) -> bool {
        self.unpromotion
    }
}

impl fmt::Display for ChessRetraction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.target)?;
        if let Some(piece) = self.uncaptured {
            write!(f, "x{}", piece)?;
        }
        write!(
            f,
            "{}{}",
            self.source,
            if self.unpromotion { "prom" } else { "" }
        )
    }
}
