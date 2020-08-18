use crate::Grammar;
use crate::Result;

use std::collections::BTreeMap;

pub trait Flatten {
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
