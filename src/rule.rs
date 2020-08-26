use crate::parser::parse_str;
use crate::Flatten;
use crate::Grammar;
use crate::Node;
use crate::Result;

use std::collections::BTreeMap;

/// Represents a single expansion that tracery can select for a key.
///
/// ```
/// # use tracery::{Grammar, Result, Rule};
/// # fn main() -> Result<()> {
/// let g = Grammar::from_json(r#"{"foo": ["bar"]}"#)?;
/// let rules = g.get_rule("foo").unwrap();
/// let expected = vec![ Rule::parse("bar")? ];
/// assert_eq!(&expected, rules);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Rule(pub(crate) Vec<Node>);

impl Rule {
    /// Create a new Rule from a Vec of individual string and tag `Node`s
    pub fn new(nodes: Vec<Node>) -> Rule {
        Rule(nodes)
    }

    /// Parses a Rule from a given string
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
