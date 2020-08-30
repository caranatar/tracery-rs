use rand::{Rng, seq::SliceRandom};
use std::collections::BTreeMap;
use std::default::Default;

use crate::{parser::parse_str, Error, Flatten, Result, Rule};

/// Represents a single grammar
///
/// This is the main data type used with this library.
pub struct Grammar {
    map: BTreeMap<String, Vec<Vec<Rule>>>,
    default_rule: String,
    modifier_registry: BTreeMap<String, Box<dyn Fn(&str) -> String>>,
}

impl Default for Grammar {
    fn default() -> Grammar {
        let modifiers = crate::modifiers::get_default_modifiers();
        Grammar {
            map: BTreeMap::new(),
            default_rule: "origin".into(),
            modifier_registry: modifiers,
        }
    }
}

impl Grammar {
    /// Create a new default `Grammar`
    pub fn new() -> Grammar {
        Default::default()
    }

    /// Gets a single modifier with the given name, if it exists
    pub fn get_modifier(&self, modifier: &str) -> Option<&dyn Fn(&str) -> String> {
        self.modifier_registry.get(modifier).map(|x| x.as_ref())
    }

    /// Gets a rule with the given key, if it exists
    pub(crate) fn get_rule(&self, key: &str) -> Option<&Vec<Rule>> {
        self.map.get(key).and_then(|stack| stack.last())
    }

    /// Creates a new grammar from a JSON grammar string
    pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
        let source: BTreeMap<String, Vec<String>> = serde_json::from_str(s.as_ref())?;
        let mut me = Grammar::new();
        for (key, value) in source.into_iter() {
            let rules: Vec<Rule> = value.iter().map(parse_str).collect::<Result<Vec<_>>>()?;
            me.map.insert(key, vec![ rules ]);
        }
        Ok(me)
    }

    /// Sets a default rule for the `Grammar`
    ///
    /// # Returns
    /// The modified `Grammar`
    pub fn default_rule<S: Into<String>>(mut self, s: S) -> Grammar {
        self.default_rule = s.into();
        self
    }

    pub fn flatten<R: ?Sized + Rng>(&self, rng: &mut R) -> Result<String> {
        match self.map.get(&self.default_rule) {
            Some(rules) => {
                let rule = rules.last().unwrap().choose(rng).unwrap();
                rule.flatten(&self, &mut BTreeMap::new(), rng)
            },
            None => Err(Error::MissingKeyError(format!(
                "Grammar does not contain key {}",
                self.default_rule
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_missing_key() -> Result<()> {
        let input = r#"{
            "a": ["a", "aa", "aaa"]
        }"#;
        let g = Grammar::from_json(input)?;
        let res = g.flatten(&mut rand::thread_rng());
        assert!(matches!(res, Err(Error::MissingKeyError(_))));

        Ok(())
    }

    #[test]
    fn set_default_rule() -> Result<()> {
        let input = r#"{
            "a": ["a", "aa", "aaa"]
        }"#;
        let g = Grammar::from_json(input)?.default_rule("a");
        let res = g.flatten(&mut rand::thread_rng())?;
        assert_eq!(res.chars().next().unwrap(), 'a');

        Ok(())
    }

    #[test]
    fn capitalize() {
        let g = Grammar::new();
        let c = g.get_modifier("capitalize").unwrap();
        assert_eq!(c(""), "");
        assert_eq!(c("a"), "A");
        assert_eq!(c("abc"), "Abc");
        assert_eq!(c("a b"), "A b");
        assert_eq!(c("aBC"), "ABC");
        assert_eq!(c("ABC"), "ABC");

        // Test expansion into multiple characters
        assert_eq!(c("ß"), "SS");
        assert_eq!(c("ßBC"), "SSBC");
        assert_eq!(c("ßbc"), "SSbc");
        assert_eq!(c("ß bc"), "SS bc");
    }

    #[test]
    fn capitalize_all() {
        let g = Grammar::new();
        let c = g.get_modifier("capitalizeAll").unwrap();
        assert_eq!(c(""), "");
        assert_eq!(c("a"), "A");
        assert_eq!(c("a b"), "A B");
        assert_eq!(c("ABC"), "ABC");
        assert_eq!(c("abc\nDEF"), "Abc\nDEF");
        assert_eq!(c("ß bc"), "SS Bc");
        assert_eq!(c("bc\t\nßßß"), "Bc\t\nSSßß");
        assert_eq!(c("\ta\nb"), "\tA\nB");
    }

    #[test]
    fn in_quotes() {
        let g = Grammar::new();
        let c = g.get_modifier("inQuotes").unwrap();
        assert_eq!(c(""), r#""""#);
        assert_eq!(c("hail eris"), r#""hail eris""#);
    }

    #[test]
    fn comma() {
        let g = Grammar::new();
        let c = g.get_modifier("comma").unwrap();

        assert_eq!(c("a,"), "a,");
        assert_eq!(c("a."), "a.");
        assert_eq!(c("a!"), "a!");
        assert_eq!(c("a?"), "a?");

        assert_eq!(c("a"), "a,");
        assert_eq!(c(""), ",");
    }

    #[test]
    fn s() {
        let g = Grammar::new();
        let c = g.get_modifier("s").unwrap();

        assert_eq!(c(""), "s");
        assert_eq!(c("harpy"), "harpies");
        assert_eq!(c("box"), "boxes");
        assert_eq!(c("index"), "indices");
        assert_eq!(c("goose"), "geese");
        assert_eq!(c("ox"), "oxen");
        assert_eq!(c("cat"), "cats");
    }

    #[test]
    fn a() {
        let g = Grammar::new();
        let c = g.get_modifier("a").unwrap();

        assert_eq!(c(""), "a ");
        assert_eq!(c("cat"), "a cat");
        assert_eq!(c("a"), "an a");
        assert_eq!(c("e"), "an e");
        assert_eq!(c("i"), "an i");
        assert_eq!(c("o"), "an o");
        assert_eq!(c("u"), "an u");
        assert_eq!(c("xylophone"), "a xylophone");
    }

    #[test]
    fn ed() {
        let g = Grammar::new();
        let c = g.get_modifier("ed").unwrap();

        assert_eq!(c(""), "");
        assert_eq!(c("box"), "boxed");
        assert_eq!(c("hail eris"), "hailed eris");
        assert_eq!(c("hail\t\neris"), "hailed\t\neris");
        assert_eq!(c("\t\nhail eris"), "\t\nhailed eris");

        assert_eq!(c("storey"), "storeyed");
        assert_eq!(c("story"), "storied");

        assert_eq!(c("blame"), "blamed");

        assert_eq!(c("\t"), "\t");
    }
}
