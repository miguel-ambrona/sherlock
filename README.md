[![Sherlock Holmes](/images/sherlock.png "Sherlock")](https://www.freepik.com/pikaso)

[![Build Status](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-ci.yml)
[![Documentation](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-docs.yml/badge.svg)](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-docs.yml)
[![Examples](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/examples.yml/badge.svg)](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/examples.yml)

# Sherlock

A chess library written in Rust, oriented to creating and solving chess
compositions with especial emphasis on retrograde analysis.

This library's name is inspired by *"The Chess Mysteries of Sherlock Holmes"*,
a master piece on retrograde analysis by the great Raymond M. Smullyan.

We rely on [jordanbray/chess](https://crates.io/crates/chess), an amazing
Rust chess library, by [Jordan Bray](https://github.com/jordanbray), for very
efficient move generation.
The images in this README and our documentation are rendered with
[web-boardimage](https://github.com/niklasf/web-boardimage), an HTTP service
developed and offered by [Niklas Fiekas](https://github.com/niklasf).
Thank you both! :heart:


## Examples

### Check the legality of a position

 Consider the following position where en-passant is possible on d3
 and all castling rights are still available:

![Example](https://backscattering.de/web-boardimage/board.svg?fen=r1bqkb1r/ppppp1pp/8/8/2pP4/8/1PP1PPPP/R1BQKB1R&arrows=Bd2d4&coordinates=true&size=300)

It turns out to be illegal!<br><br>

<details>
<summary>Click here for an explanation.<br><br></summary>
First, realize that only knights and pawns can have moved in this game.
Then, observe that for the black F-pawn to reach c4, it must have captured white
knights on e6 and d5, and also the white A-pawn on c4 (who reached this square by
capturing black knights on b3 and c4).

This makes it possible to determine the parity of the number of moves performed
by each side.
 * White made an **even** number of moves: 3 pawn moves and an odd
 number of knight moves, since the white knights finished the game on squares of the
 same color.

 * Black also made an **even** number of moves: 3 pawn moves and
 again, an odd number of knight moves.

 Since both players made an even number of moves, they must have moved the same
 number of times and it should be White's turn, but it is not!
</details>
Sherlock can realize this.<br><br>

```rust
use chess::Board;
use std::str::FromStr;

let board = Board::from_str("r1bqkb1r/ppppp1pp/8/8/2pP4/8/1PP1PPPP/R1BQKB1R b KQkq d3").unwrap();
assert_eq!(sherlock::is_legal(&board), false);

// the same position with en-passant disabled would be legal
let board = Board::from_str("r1bqkb1r/ppppp1pp/8/8/2pP4/8/1PP1PPPP/R1BQKB1R b KQkq -").unwrap();
assert_eq!(sherlock::is_legal(&board), true);
```

### The Mystery of the Missing Piece

This is a composition by Raymond M. Smullyan from the above-mentioned book.
On h4 rests a shilling instead of a chess piece. The challenge is to determine
what piece it is.

![Example](https://backscattering.de/web-boardimage/board.svg?fen=2nR3K/pk1Rp1p1/p2p4/P1p5/1Pp5/2PP2P1/4P2P/n7&squares=h4&coordinates=true&size=300)

Sherlock can realize that the shilling must be a *white bishop*
(refer to the book for an explanation):

```rust
use chess::{Board, Color::White, Piece::Bishop, Square};
use std::str::FromStr;
use sherlock::{is_legal, ALL_COLORED_PIECES};

let board = Board::from_str("2nR3K/pk1Rp1p1/p2p4/P1p5/1Pp5/2PP2P1/4P2P/n7 b - -").unwrap();

// all the pieces that lead to a legal position when placed on h4
let valid_pieces: Vec<_> = ALL_COLORED_PIECES
    .into_iter()
    .filter(|&(color, piece)| {
        board.set_piece(piece, color, Square::H4).as_ref().map_or(false, is_legal)
    })
    .collect();

assert_eq!(valid_pieces, vec![(White, Bishop)]);
```
