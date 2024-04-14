[![Sherlock Holmes with City Background](/images/sherlock.png "Sherlock")](https://www.freepik.com/pikaso)

[![Build Status](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-ci.yml)
[![Documentation](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-docs.yml/badge.svg)](https://github.com/miguel-ambrona/sherlock-rust/actions/workflows/rust-docs.yml)

# Sherlock

A chess library written in Rust, oriented to creating and solving chess 
compositions with especial emphasis on retrograde analysis.

We rely on [jordanbray/chess](https://crates.io/crates/chess), an amazing
Rust chess library for very efficient move generation.

## Examples

### Check the legality of a position

```rust
use chess::Board;
use sherlock::is_legal;

let board = Board::default();
assert!(is_legal(&board));
```