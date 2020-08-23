use crate::{grammar::Grammar, parser::parse_tag, Error, Flatten, Result, Rule};
use rand::seq::SliceRandom;
use std::collections::BTreeMap;

/// Structure representing a `#tag#` in a tracery rule
///
/// The `key` represents the name of the rule to look for replacements from.
/// The `actions` are bracketed and come before the `key', like `[action:#otherkey#]key`
/// The `modifiers` change the result of replacing `key`. Some examples are:
///
/// ```ignore
/// #key.s# // => pluralizes the replacement
/// #key.capitalize# // => capitalizes the first word of the replacement
/// #key.inQuotes# // => wraps the replacement in quotes
/// // etc...
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    pub(crate) key: String,
    pub(crate) actions: BTreeMap<String, Rule>,
    pub(crate) modifiers: Vec<String>,
}

impl Tag {
    pub fn new<S: Into<String>>(key: S) -> Tag {
        Tag {
            key: key.into(),
            actions: BTreeMap::new(),
            modifiers: vec![],
        }
    }

    /// uses self.key to retrieve a list of rules for that key.
    /// First we look in the `Tag`s `actions`, and if the key isn't present, we use a `Grammar`
    /// (presumably the context that we are flattening this tag in)
    pub fn get_rule(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
    ) -> Result<String> {
        if let Some(rule) = overrides.get(&self.key) {
            return Ok(rule.clone());
        }

        if let Some(rules) = grammar.get_rule(&self.key) {
            let mut rng = rand::thread_rng();
            let choice = rules.choose(&mut rng).unwrap();
            return choice.flatten(grammar, overrides);
        }

        Err(Error::MissingKeyError(format!(
            "Could not find key {}",
            self.key
        )))
    }

    pub fn apply_modifiers(&self, s: &str, grammar: &Grammar) -> String {
        let mut string = String::from(s);
        for modifier in self.modifiers.iter() {
            if let Some(f) = grammar.get_modifier(modifier) {
                string = f(&string);
            }
        }
        string
    }

    pub fn with_actions(mut self, actions: BTreeMap<String, Rule>) -> Tag {
        self.actions = actions;
        self
    }

    pub fn with_modifiers<S: Into<String>>(mut self, modifiers: Vec<S>) -> Tag {
        self.modifiers = modifiers.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn parse(s: &str) -> Result<Tag> {
        Ok(parse_tag(s)?)
    }
}

impl Flatten for Tag {
    fn flatten(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
    ) -> Result<String> {
        let mut map = BTreeMap::new();
        for (label, rule) in self.actions.clone().into_iter() {
            map.insert(label, rule.flatten(grammar, overrides)?);
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

        let choice = self.get_rule(grammar, &mut overrides)?;

        let modified = self.apply_modifiers(&choice, grammar);

        Ok(modified)
    }
}

#[cfg(test)]
mod tests {}
