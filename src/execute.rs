use crate::Grammar;
use crate::Result;

use rand::Rng;

/// A trait for types that can be flattened into an output string
pub trait Execute {
    /// Given a grammar and a set of overriden rules (from actions), produces a
    /// single "flattened" output string or an error
    fn execute<R: ?Sized + Rng>(&self, grammar: &mut Grammar, rng: &mut R) -> Result<String>;
}
