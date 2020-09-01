use crate::Grammar;
use crate::Result;

use rand::Rng;

use std::collections::BTreeMap;

/// A trait for types that can be flattened into an output string
pub trait Flatten {
    /// Given a grammar and a set of overriden rules (from actions), produces a
    /// single "flattened" output string or an error
    fn flatten<R: ?Sized + Rng>(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
        rng: &mut R
    ) -> Result<String>;
}
