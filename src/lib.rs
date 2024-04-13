//! # Sherlock
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

mod rules;
mod squares;
mod state;
mod util;

mod legality;
pub use crate::legality::*;
