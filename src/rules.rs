use std::fmt;

use crate::analysis::Analysis;

/// A legality rule, it updates the analysis on the legality of the position,
/// after deriving new information.
pub trait Rule: fmt::Debug {
    /// Initializes the rule state for a given board.
    fn new() -> Self
    where
        Self: Sized + fmt::Debug;

    /// Update the rule's state based on the given analysis state.
    fn update(&mut self, analysis: &Analysis);

    /// Check whether or not it makes sense to apply the rule (we do not want to
    /// apply a rule if we are sure it will not derive any new information).
    fn is_applicable(&self, analysis: &Analysis) -> bool;

    /// Applies the rule, possibly modifying the legality analysis after having
    /// derived new information.
    /// Returns `true` iff progress has been made.
    fn apply(&self, analysis: &mut Analysis) -> bool;
}

mod material;
pub use material::*;

mod steady;
pub use steady::*;

mod origins;
pub use origins::*;

mod refine_origins;
pub use refine_origins::*;

mod destinies;
pub use destinies::*;

mod route_from_origins;
pub use route_from_origins::*;

mod route_to_destinies;
pub use route_to_destinies::*;

mod captures_bounds;
pub use captures_bounds::*;
