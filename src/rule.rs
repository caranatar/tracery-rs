use crate::Flatten;
use crate::Grammar;
use crate::Node;
use crate::Result;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Rule(pub(crate) Vec<Node>);

impl Rule {
    pub(crate) fn new(nodes: Vec<Node>) -> Rule {
        Rule(nodes)
    }
}

impl Flatten for Rule {
    fn flatten<R: ?Sized + rand::Rng>(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
        rng: &mut R,
    ) -> Result<String> {
        let parts = self
            .0
            .iter()
            .map(|n| n.flatten(grammar, overrides, rng))
            .collect::<Result<Vec<String>>>()?;
        Ok(parts.join(""))
    }
}
