use crate::{grammar::Grammar, Error, Flatten, Result, Rule};
use rand::{Rng, seq::SliceRandom};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Tag {
    pub(crate) key: String,
    pub(crate) actions: BTreeMap<String, Rule>,
    pub(crate) modifiers: Vec<String>,
}

impl Tag {
    /// Creates a tag with the given key and no associated actions or modifiers
    pub(crate) fn new<S: Into<String>>(key: S) -> Tag {
        Tag {
            key: key.into(),
            actions: BTreeMap::new(),
            modifiers: vec![],
        }
    }

    /// uses self.key to retrieve a list of rules for that key.
    /// First we look in the `Tag`s `actions`, and if the key isn't present, we use a `Grammar`
    /// (presumably the context that we are flattening this tag in)
    pub(crate) fn get_rule<R: ?Sized + Rng>(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
        rng: &mut R
    ) -> Result<String> {
        if let Some(rule) = overrides.get(&self.key) {
            return Ok(rule.clone());
        }

        if let Some(rules) = grammar.get_rule(&self.key) {
            let choice = rules.choose(rng).unwrap();
            return choice.flatten(grammar, overrides, rng);
        }

        Err(Error::MissingKeyError(format!(
            "Could not find key {}",
            self.key
        )))
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
    pub(crate) fn with_actions(mut self, actions: BTreeMap<String, Rule>) -> Tag {
        self.actions = actions;
        self
    }

    /// Adds the given modifiers to this tag
    pub(crate) fn with_modifiers<S: Into<String>>(mut self, modifiers: Vec<S>) -> Tag {
        self.modifiers = modifiers.into_iter().map(|s| s.into()).collect();
        self
    }
}

impl Flatten for Tag {
    fn flatten<R: ?Sized + Rng>(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
        rng: &mut R,
    ) -> Result<String> {
        let mut map = BTreeMap::new();
        for (label, rule) in self.actions.clone().into_iter() {
            map.insert(label, rule.flatten(grammar, overrides, rng)?);
        }

        // all children of this node need to have an `overrides` map with the
        // values obtained from flattening the actions, so get our own copy of
        // `overrides` here, use it to flatten the children, then let it go out of
        // scope when this is done
        let mut overrides = overrides.clone();

        // merge `map` and `overrides` now, overwriting anything in `overrides` with what
        // we obtained and put into `map`
        for (label, rule) in map.into_iter() {
            overrides.insert(label, rule);
        }

        let choice = self.get_rule(grammar, &mut overrides, rng)?;

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
        let input = hashmap!{ "a" => vec!["b"] };
        let g = Grammar::from_map(input)?;
        let tag = parse_tag("#a#")?;
        let r = tag.get_rule(&g, &mut BTreeMap::new(), &mut rand::thread_rng())?;
        assert_eq!(r, "b");
        Ok(())
    }

    #[test]
    fn get_rule_from_overrides() -> Result<()> {
        let input = hashmap!{ "a" => vec!["b"] };
        let g = Grammar::from_map(input)?;
        let tag = parse_tag("#a#")?;
        let mut overrides = BTreeMap::new();
        overrides.insert("a".to_string(), "c".to_string());
        overrides.insert("b".to_string(), "d".to_string());
        let r = tag.get_rule(&g, &mut overrides, &mut rand::thread_rng())?;
        assert_eq!(r, "c");
        let tag = parse_tag("#b#")?;
        let r = tag.get_rule(&g, &mut overrides, &mut rand::thread_rng())?;
        assert_eq!(r, "d");
        Ok(())
    }

    #[test]
    fn get_rule_missing_key() -> Result<()> {
        let input = hashmap!{ "a" => vec!["b"] };
        let g = Grammar::from_map(input)?;
        let tag = parse_tag("#b#")?;
        let r = tag.get_rule(&g, &mut BTreeMap::new(), &mut rand::thread_rng());
        assert!(matches!(r, Err(Error::MissingKeyError(_))));
        Ok(())
    }

    #[test]
    fn apply_modifiers() -> Result<()> {
        let input = hashmap!{ "a" => vec!["b"] };
        let g = Grammar::from_map(input)?;
        let tag = parse_tag("#b.capitalize#")?;
        let x = tag.apply_modifiers("x", &g);
        assert_eq!(x, "X");
        Ok(())
    }
}
