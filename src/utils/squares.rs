#![allow(unused)]

use chess::{BitBoard, Square, ALL_SQUARES, EMPTY};

/// The light squares of the chess board.
pub const LIGHT_SQUARES: BitBoard = BitBoard(6172840429334713770);

/// The dark squares of the chess board.
pub const DARK_SQUARES: BitBoard = BitBoard(12273903644374837845);

/// The light and dark squares of the chess board.
pub const COLOR_SQUARES: [BitBoard; 2] = [LIGHT_SQUARES, DARK_SQUARES];

/// The A1 square on the chess board.
pub const A1: Square = Square::A1;

/// The A2 square on the chess board.
pub const A2: Square = Square::A2;

/// The A3 square on the chess board.
pub const A3: Square = Square::A3;

/// The A4 square on the chess board.
pub const A4: Square = Square::A4;

/// The A5 square on the chess board.
pub const A5: Square = Square::A5;

/// The A6 square on the chess board.
pub const A6: Square = Square::A6;

/// The A7 square on the chess board.
pub const A7: Square = Square::A7;

/// The A8 square on the chess board.
pub const A8: Square = Square::A8;

/// The B1 square on the chess board.
pub const B1: Square = Square::B1;

/// The B2 square on the chess board.
pub const B2: Square = Square::B2;

/// The B3 square on the chess board.
pub const B3: Square = Square::B3;

/// The B4 square on the chess board.
pub const B4: Square = Square::B4;

/// The B5 square on the chess board.
pub const B5: Square = Square::B5;

/// The B6 square on the chess board.
pub const B6: Square = Square::B6;

/// The B7 square on the chess board.
pub const B7: Square = Square::B7;

/// The B8 square on the chess board.
pub const B8: Square = Square::B8;

/// The C1 square on the chess board.
pub const C1: Square = Square::C1;

/// The C2 square on the chess board.
pub const C2: Square = Square::C2;

/// The C3 square on the chess board.
pub const C3: Square = Square::C3;

/// The C4 square on the chess board.
pub const C4: Square = Square::C4;

/// The C5 square on the chess board.
pub const C5: Square = Square::C5;

/// The C6 square on the chess board.
pub const C6: Square = Square::C6;

/// The C7 square on the chess board.
pub const C7: Square = Square::C7;

/// The C8 square on the chess board.
pub const C8: Square = Square::C8;

/// The D1 square on the chess board.
pub const D1: Square = Square::D1;

/// The D2 square on the chess board.
pub const D2: Square = Square::D2;

/// The D3 square on the chess board.
pub const D3: Square = Square::D3;

/// The D4 square on the chess board.
pub const D4: Square = Square::D4;

/// The D5 square on the chess board.
pub const D5: Square = Square::D5;

/// The D6 square on the chess board.
pub const D6: Square = Square::D6;

/// The D7 square on the chess board.
pub const D7: Square = Square::D7;

/// The D8 square on the chess board.
pub const D8: Square = Square::D8;

/// The E1 square on the chess board.
pub const E1: Square = Square::E1;

/// The E2 square on the chess board.
pub const E2: Square = Square::E2;

/// The E3 square on the chess board.
pub const E3: Square = Square::E3;

/// The E4 square on the chess board.
pub const E4: Square = Square::E4;

/// The E5 square on the chess board.
pub const E5: Square = Square::E5;

/// The E6 square on the chess board.
pub const E6: Square = Square::E6;

/// The E7 square on the chess board.
pub const E7: Square = Square::E7;

/// The E8 square on the chess board.
pub const E8: Square = Square::E8;

/// The F1 square on the chess board.
pub const F1: Square = Square::F1;

/// The F2 square on the chess board.
pub const F2: Square = Square::F2;

/// The F3 square on the chess board.
pub const F3: Square = Square::F3;

/// The F4 square on the chess board.
pub const F4: Square = Square::F4;

/// The F5 square on the chess board.
pub const F5: Square = Square::F5;

/// The F6 square on the chess board.
pub const F6: Square = Square::F6;

/// The F7 square on the chess board.
pub const F7: Square = Square::F7;

/// The F8 square on the chess board.
pub const F8: Square = Square::F8;

/// The G1 square on the chess board.
pub const G1: Square = Square::G1;

/// The G2 square on the chess board.
pub const G2: Square = Square::G2;

/// The G3 square on the chess board.
pub const G3: Square = Square::G3;

/// The G4 square on the chess board.
pub const G4: Square = Square::G4;

/// The G5 square on the chess board.
pub const G5: Square = Square::G5;

/// The G6 square on the chess board.
pub const G6: Square = Square::G6;

/// The G7 square on the chess board.
pub const G7: Square = Square::G7;

/// The G8 square on the chess board.
pub const G8: Square = Square::G8;

/// The H1 square on the chess board.
pub const H1: Square = Square::H1;

/// The H2 square on the chess board.
pub const H2: Square = Square::H2;

/// The H3 square on the chess board.
pub const H3: Square = Square::H3;

/// The H4 square on the chess board.
pub const H4: Square = Square::H4;

/// The H5 square on the chess board.
pub const H5: Square = Square::H5;

/// The H6 square on the chess board.
pub const H6: Square = Square::H6;

/// The H7 square on the chess board.
pub const H7: Square = Square::H7;

/// The H8 square on the chess board.
pub const H8: Square = Square::H8;
