use crate::Execute;
use crate::Grammar;
use crate::Node;
use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Rule(pub(crate) Vec<Node>);

impl Rule {
    pub(crate) fn new(nodes: Vec<Node>) -> Rule {
        Rule(nodes)
    }
}

impl Execute for Rule {
    fn execute<R: ?Sized + rand::Rng>(
        &self,
        grammar: &mut Grammar,
        rng: &mut R
    ) -> Result<String> {
        let parts = self
            .0
            .iter()
            .map(|n| n.execute(grammar, rng))
            .collect::<Result<Vec<String>>>()?;
        Ok(parts.join(""))
    }
}
