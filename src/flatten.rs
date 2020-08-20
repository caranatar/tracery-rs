use crate::Grammar;
use crate::Result;

use std::collections::BTreeMap;

/// A trait for types that can be flattened into an output string
pub trait Flatten {
    /// Given a grammar and a set of overriden rules (from actions), produces a
    /// single "flattened" output string or an error
    fn flatten(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
    ) -> Result<String>;
}

impl Flatten for String {
    fn flatten(&self, _: &Grammar, _: &mut BTreeMap<String, String>) -> Result<String> {
        Ok(self.to_owned())
    }
}
