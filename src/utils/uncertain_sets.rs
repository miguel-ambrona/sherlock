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

    fn simplify(&mut self) {
        let all_elements = self.certain | self.candidates;
        if all_elements.popcnt() == self.size {
            self.certain = all_elements;
            self.candidates = EMPTY;
        }
    }
}

impl fmt::Display for UncertainSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "size: {}", self.size)?;
        writeln!(f, "certain:\n{}", self.certain)?;
        writeln!(f, "candidates:\n{}", self.candidates)
    }
}
