use std::fmt;

use chess::{BitBoard, EMPTY};

#[derive(Debug, Clone, Copy)]
pub struct UncertainSet {
    /// Size of the set.
    size: u32,
    /// Elements that are certainly in the set.
    certain: BitBoard,
    /// Elements that may be in the set.
    candidates: BitBoard,
}

impl UncertainSet {
    pub fn new(size: u32) -> Self {
        UncertainSet {
            size,
            certain: EMPTY,
            candidates: !EMPTY,
        }
    }

    /// Adds the given elements to Self.
    /// Returns `true` iff this operation modified Self.
    pub fn add(&mut self, set: BitBoard) -> bool {
        let new_certain = self.certain | set;
        if new_certain == self.certain {
            return false;
        }
        self.certain = new_certain;
        self.candidates &= !self.certain;
        self.simplify();
        true
    }

    /// Removes the given elements from Self.
    /// Returns `true` iff this operation modified Self.
    pub fn remove(&mut self, set: BitBoard) -> bool {
        let new_candidates = self.candidates & !set;
        if new_candidates == self.candidates {
            return false;
        }
        self.candidates = new_candidates;
        self.simplify();
        true
    }

    fn simplify(&mut self) {
        if (self.certain | self.candidates).popcnt() == self.size {
            self.certain |= self.candidates;
        }

        if self.certain.popcnt() == self.size {
            self.candidates = EMPTY;
        }
    }
}

impl fmt::Display for UncertainSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "size: {}", self.size)?;
        writeln!(f, "certain:\n{}", self.certain.reverse_colors())?;
        writeln!(f, "candidates:\n{}", self.candidates.reverse_colors())
    }
}
