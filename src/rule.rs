use crate::Execute;
use crate::Grammar;
use crate::Node;
use crate::Result;

use lazy_static::lazy_static;

lazy_static! {
    static ref POP: String = String::from("POP");
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Rule(pub(crate) Vec<Node>);

impl Rule {
    pub(crate) fn new(nodes: Vec<Node>) -> Rule {
        Rule(nodes)
    }

    pub(crate) fn is_pop(&self) -> bool {
        self.0.len() == 1 &&
            self.0.first().unwrap().text() == Some(&POP)
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
