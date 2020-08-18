use crate::Flatten;
use crate::Grammar;
use crate::Node;
use crate::parser::parse_str;
use crate::Result;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Rule(pub(crate) Vec<Node>);

impl Rule {
    pub fn new(nodes: Vec<Node>) -> Rule {
        Rule(nodes)
    }

    pub fn parse<S: AsRef<str>>(source: S) -> Result<Rule> {
        parse_str(source.as_ref())
    }
}

impl Flatten for Rule {
    fn flatten(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
    ) -> Result<String> {
        let parts = self
            .0
            .iter()
            .map(|n| n.flatten(grammar, overrides))
            .collect::<Result<Vec<String>>>()?;
        Ok(parts.join(""))
    }
}
