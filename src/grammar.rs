use rand::{seq::SliceRandom, Rng};
use std::collections::BTreeMap;
use std::default::Default;
use std::rc::Rc;

use crate::{parser::parse_str, Error, Execute, Result, Rule};

/// Represents a single grammar
///
/// This is the main data type used with this library.
#[derive(Clone)]
pub struct Grammar {
    map: BTreeMap<String, Vec<Vec<Rule>>>,
    default_rule: String,
    modifier_registry: BTreeMap<String, Rc<dyn Fn(&str) -> String>>,
}

impl Default for Grammar {
    fn default() -> Grammar {
        Grammar {
            map: BTreeMap::new(),
            default_rule: "origin".into(),
            modifier_registry: crate::modifiers::get_default_modifiers(),
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

    /// Pushes a new rule onto the rule stack for a given key
    pub(crate) fn push_rule(&mut self, key: String, rule_str: String) {
        use crate::Node;
        use std::collections::btree_map::Entry;
        let rule = vec![Rule::new(vec![Node::from(rule_str)])];
        match self.map.entry(key) {
            Entry::Occupied(mut occ) => {
                let stack = occ.get_mut();
                stack.push(rule);
            }
            Entry::Vacant(vac) => {
                vac.insert(vec![rule]);
            }
        }
    }

    /// Pops a rule off the rule stack for a given key, removing the key
    /// entirely if there are no rules left
    pub(crate) fn pop_rule(&mut self, key: String) {
        use std::collections::btree_map::Entry;
        if let Entry::Occupied(mut occ) = self.map.entry(key) {
            let stack = occ.get_mut();
            if stack.len() < 2 {
                occ.remove_entry();
            } else {
                stack.pop();
            }
        }
    }

    /// Gets a rule with the given key, if it exists
    pub(crate) fn get_rule(&self, key: &str) -> Option<&Vec<Rule>> {
        self.map.get(key).and_then(|stack| stack.last())
    }

    /// Creates a new grammar from a JSON grammar string
    #[cfg(feature = "tracery_json")]
    pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
        let source: BTreeMap<String, Vec<String>> = serde_json::from_str(s.as_ref())?;
        let mut me = Grammar::new();
        for (key, value) in source.into_iter() {
            let rules: Vec<Rule> = value.iter().map(parse_str).collect::<Result<Vec<_>>>()?;
            me.map.insert(key, vec![rules]);
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

    /// Attempts to use the Grammar to produce an output String.
    ///
    /// This method clones the Grammar, so any changes made in the course of
    /// producing an output string (such as pushing a new rule onto a stack
    /// using a labeled action such as `[foo:bar]`) will be discarded after
    /// the output is produced.
    ///
    /// If you wish to preserve changes use [`execute`]
    ///
    /// [`execute`]: struct.Grammar.html#method.execute
    pub fn flatten<R: ?Sized + Rng>(&self, rng: &mut R) -> Result<String> {
        self.clone().execute(&self.default_rule, rng)
    }

    /// Attempts to use the Grammar to produce an output String, preserving any
    /// side effects that occur while doing so.
    ///
    /// This method produces an output string, but preserves any changes made to
    /// the Grammar in the course of doing so. For instance, if a labeled action
    /// such as `[foo:bar]` is executed, then the Grammar will maintain that
    /// rule after this method returns.
    ///
    /// If you wish to produce an output String without preserving changes, used
    /// [`flatten`].
    ///
    /// [`flatten`]: struct.Grammar.html#method.flatten
    pub fn execute<R>(&mut self, key: &String, rng: &mut R) -> Result<String>
    where
        R: ?Sized + Rng,
    {
        let rule = match self.map.get(key) {
            Some(rules) => Ok(rules.last().unwrap().choose(rng).unwrap().clone()),
            None => Err(Error::MissingKeyError(format!(
                "Grammar does not contain key {}",
                self.default_rule
            ))),
        }?;
        rule.execute(self, rng)
    }

    /// Creates a new Grammar from an input map of keys to rule lists
    ///
    /// # Notes
    /// Any object implementing
    /// `IntoIterator<Item = (Into<String>, Into<Vec<Into<String>>>)>` will be
    /// accepted by this function, despite its name
    pub fn from_map<I, K, C, S>(iter: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, C)>,
        K: Into<String>,
        C: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut map: BTreeMap<String, Vec<Vec<Rule>>> = BTreeMap::new();

        for (k, v) in iter {
            let rules: Vec<Rule> = v
                .into_iter()
                .map(|x| parse_str(x.into()))
                .collect::<Result<Vec<_>>>()?;
            map.insert(k.into(), vec![rules]);
        }

        Ok(Grammar {
            map,
            default_rule: String::from("origin"),
            modifier_registry: crate::modifiers::get_default_modifiers(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn flatten_missing_key() -> Result<()> {
        let input = hashmap! {
            "a" => vec![ "a", "aa", "aaa" ]
        };
        let g = Grammar::from_map(input)?;
        let res = g.flatten(&mut rand::thread_rng());
        assert!(matches!(res, Err(Error::MissingKeyError(_))));

        Ok(())
    }

    #[test]
    fn set_default_rule() -> Result<()> {
        let input = hashmap! {
            "a" => vec![ "a", "aa", "aaa" ]
        };
        let g = Grammar::from_map(input)?.default_rule("a");
        let res = g.flatten(&mut rand::thread_rng())?;
        assert_eq!(res.chars().next().unwrap(), 'a');

        Ok(())
    }

    #[test]
    #[cfg(feature = "tracery_json")]
    fn from_json() -> Result<()> {
        let x = r#"{
            "origin": [ "a", "aa" ]
        }"#;
        let g = Grammar::from_json(x)?;
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

    #[test]
    fn execute() -> Result<()> {
        let input = hashmap!{
            "origin" => vec!["#[foo:bar]baz#"],
            "baz" => vec!["baz"]
        };
        let mut grammar = Grammar::from_map(input)?;

        // The first invocation should produce the string baz from the rule baz
        let origin = String::from("origin");
        assert_eq!("baz", grammar.execute(&origin, &mut rand::thread_rng())?);

        // It should have also produced a new rule foo with the value bar
        let origin = String::from("foo");
        assert_eq!("bar", grammar.execute(&origin, &mut rand::thread_rng())?);

        Ok(())
    }

    #[test]
    fn execute_function() -> Result<()> {
        let input = hashmap!{
            "origin" => vec!["#setFoo##baz#"],
            "setFoo" => vec!["[foo:bar][bar:#[qux:quux]baz#]"],
            "baz" => vec!["baz"]
        };
        let mut grammar = Grammar::from_map(input)?;

        // The first invocation should produce the string baz from the rule baz
        let origin = String::from("origin");
        assert_eq!("baz", grammar.execute(&origin, &mut rand::thread_rng())?);

        // It should have also produced a new rule foo with the value bar
        let origin = String::from("foo");
        assert_eq!("bar", grammar.execute(&origin, &mut rand::thread_rng())?);

        // ..and a new rule bar with the value baz
        let origin = String::from("bar");
        assert_eq!("baz", grammar.execute(&origin, &mut rand::thread_rng())?);

        // ..aaaand a new rule qux with the value quux
        let origin = String::from("qux");
        assert_eq!("quux", grammar.execute(&origin, &mut rand::thread_rng())?);

        Ok(())
    }

    #[test]
    fn pop_rule() -> Result<()> {
        let input = hashmap! {
            "origin" => vec!["#[foo:baz]foo##[foo:POP]foo#"],
            "foo" => vec!["bar"]
        };
        let mut grammar = Grammar::from_map(input)?;
        assert_eq!("bazbar", grammar.execute(&String::from("origin"), &mut rand::thread_rng())?);
        Ok(())
    }

    #[test]
    fn pop_and_remove() -> Result<()> {
        let input = hashmap! {
            "origin" => vec!["#foo##popFoo#"],
            "foo" => vec!["bar"],
            "popFoo" => vec!["[foo:POP]"]
        };
        let mut grammar = Grammar::from_map(input)?;
        let mut rng = rand::thread_rng();
        let origin = String::from("origin");
        assert_eq!("bar", grammar.execute(&origin, &mut rng)?);
        let origin = String::from("foo");
        assert!(matches!(grammar.execute(&origin, &mut rng), Err(Error::MissingKeyError(_))));
        Ok(())
    }
}
