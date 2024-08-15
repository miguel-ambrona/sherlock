[![Sherlock Holmes](/images/sherlock.png "Sherlock")](https://www.freepik.com/pikaso)

[![Build Status](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-ci.yml)
[![Documentation](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-docs.yml/badge.svg)](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-docs.yml)

# Sherlock

A chess library written in Rust, oriented to creating and solving chess
compositions with especial emphasis on retrograde analysis.

This library's name is inspired by *"The Chess Mysteries of Sherlock Holmes"*,
a master piece on retrograde analysis by the great Raymond Smullyan.

We rely on [jordanbray/chess](https://crates.io/crates/chess), an amazing
Rust chess library for very efficient move generation.

## Examples

### Check the legality of a position

 <details open>
 <summary>Consider the following position where en-passant is possible on d3
 and all castling rights are still available:<br><br></summary>

![Example](https://backscattering.de/web-boardimage/board.svg?fen=r1bqkb1r/ppppp1pp/8/8/2pP4/8/1PP1PPPP/R1BQKB1R&arrows=Bd2d4)

</details>
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
use sherlock::is_legal;
use std::str::FromStr;

let board = Board::from_str("r1bqkb1r/ppppp1pp/8/8/2pP4/8/1PP1PPPP/R1BQKB1R b KQkq d3").unwrap();
assert_eq!(is_legal(&board), false);

// the same position with en-passant disabled would be legal
let board = Board::from_str("r1bqkb1r/ppppp1pp/8/8/2pP4/8/1PP1PPPP/R1BQKB1R b KQkq -").unwrap();
assert_eq!(is_legal(&board), true);
```
