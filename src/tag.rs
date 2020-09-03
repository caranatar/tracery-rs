use crate::{grammar::Grammar, Error, Execute, Result, Rule};
use rand::{seq::SliceRandom, Rng};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Action {
    pub(crate) label: Option<String>,
    pub(crate) rule: Rule,
}

impl From<(Option<String>, Rule)> for Action {
    fn from((label, rule): (Option<String>, Rule)) -> Self {
        Action {
            label,
            rule,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Tag {
    pub(crate) key: Option<String>,
    pub(crate) actions: Vec<Action>,
    pub(crate) modifiers: Vec<String>,
}

impl Tag {
    /// Creates a tag with the given key and no associated actions or modifiers
    pub(crate) fn new<S: Into<String>>(key: S) -> Tag {
        Tag {
            key: Some(key.into()),
            actions: Vec::new(),
            modifiers: Vec::new(),
        }
    }

    pub(crate) fn empty() -> Tag {
        Tag {
            key: None,
            actions: Vec::new(),
            modifiers: Vec::new(),
        }
    }

    pub(crate) fn get_rule<R: ?Sized + Rng>(
        &self,
        grammar: &mut Grammar,
        rng: &mut R,
    ) -> Result<String> {
        match &self.key {
            Some(key) => {
                let rule = match grammar.get_rule(key) {
                    Some(rules) => Ok(rules.choose(rng).unwrap().clone()),
                    None => Err(Error::MissingKeyError(format!(
                        "Could not find key {}",
                        key
                    ))),
                }?;
                rule.execute(grammar, rng)
            }
            None => Ok(String::default()),
        }
    }

    /// Applies the modifiers associated with this Tag to a given string, using
    /// the definitions in the given Grammar
    pub(crate) fn apply_modifiers(&self, s: &str, grammar: &Grammar) -> String {
        let mut string = String::from(s);
        for modifier in self.modifiers.iter() {
            if let Some(f) = grammar.get_modifier(modifier) {
                string = f(&string);
            }
        }
        string
    }

    /// Adds the given actions to this tag
    pub(crate) fn with_actions<A>(mut self, mut actions: Vec<A>) -> Tag
    where
        A: Into<Action>
    {
        self.actions = actions.drain(..).map(|a| a.into()).collect();
        self
    }

    /// Adds the given modifiers to this tag
    pub(crate) fn with_modifiers<S: Into<String>>(mut self, modifiers: Vec<S>) -> Tag {
        self.modifiers = modifiers.into_iter().map(|s| s.into()).collect();
        self
    }
}

impl Execute for Tag {
    fn execute<R: ?Sized + Rng>(&self, grammar: &mut Grammar, rng: &mut R) -> Result<String> {
        for action in &self.actions {
            if action.rule.is_pop() && action.label.is_some() {
                grammar.pop_rule(action.label.as_ref().unwrap().clone());
            } else {
                let output = action.rule.execute(grammar, rng)?;
                if action.label.is_some() {
                    grammar.push_rule(action.label.as_ref().unwrap().clone(), output);
                }
            }
        }

        let choice = self.get_rule(grammar, rng)?;

        let modified = self.apply_modifiers(&choice, grammar);

        Ok(modified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_tag;
    use maplit::hashmap;

    #[test]
    fn get_rule_from_grammar() -> Result<()> {
        let input = hashmap! { "a" => vec!["b"] };
        let mut g = Grammar::from_map(input)?;
        let tag = parse_tag("#a#")?;
        let r = tag.get_rule(&mut g, &mut rand::thread_rng())?;
        assert_eq!(r, "b");
        Ok(())
    }

    #[test]
    fn get_rule_missing_key() -> Result<()> {
        let input = hashmap! { "a" => vec!["b"] };
        let mut g = Grammar::from_map(input)?;
        let tag = parse_tag("#b#")?;
        let r = tag.get_rule(&mut g, &mut rand::thread_rng());
        assert!(matches!(r, Err(Error::MissingKeyError(_))));
        Ok(())
    }

    #[test]
    fn apply_modifiers() -> Result<()> {
        let input = hashmap! { "a" => vec!["b"] };
        let g = Grammar::from_map(input)?;
        let tag = parse_tag("#b.capitalize#")?;
        let x = tag.apply_modifiers("x", &g);
        assert_eq!(x, "X");
        Ok(())
    }
}
