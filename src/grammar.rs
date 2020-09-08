use lazy_static::lazy_static;
use rand::{seq::SliceRandom, Rng};
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::{parser::parse_str, Error, Execute, Result, Rule};

lazy_static! {
    pub(crate) static ref ORIGIN: String = String::from("origin");
}

/// Represents a single, complete tracery grammar.
///
/// See the [`crate-level documentation`] for a usage overview.
///
/// [`crate-level documentation`]: index.html
#[derive(Clone)]
pub struct Grammar {
    map: BTreeMap<String, Vec<Vec<Rule>>>,
    default_rule: String,
    modifier_registry: BTreeMap<String, Rc<dyn Fn(&str) -> String>>,
}

impl Grammar {
    pub(crate) fn get_modifier(&self, modifier: &str) -> Option<&dyn Fn(&str) -> String> {
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
    ///
    /// # Examples
    /// ```
    /// use tracery::Grammar;
    /// # use tracery::Result;
    /// # use maplit::hashmap;
    /// # fn main() -> Result<()> {
    /// let json = r##"{
    ///     "origin": [ "#tool# is #description#!" ],
    ///     "tool": [ "tracery" ],
    ///     "description": [ "fun", "awesome" ]
    /// }"##;
    /// let g = Grammar::from_json(json)?;
    /// # let output = g.flatten(&mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Error`]: enum.Error.html
    /// [`Grammar`]: struct.Grammar.html
    #[cfg(feature = "tracery_json")]
    pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
        let source: BTreeMap<String, Vec<String>> = serde_json::from_str(s.as_ref())?;
        let mut map: BTreeMap<String, Vec<Vec<Rule>>> = BTreeMap::new();
        for (key, value) in source.into_iter() {
            let rules: Vec<Rule> = value.iter().map(parse_str).collect::<Result<Vec<_>>>()?;
            map.insert(key, vec![rules]);
        }

        Ok(Grammar {
            map,
            default_rule: ORIGIN.clone(),
            modifier_registry: crate::modifiers::get_default_modifiers(),
        })
    }

    /// Sets a default rule, then returns the modified Grammar
    ///
    /// # Examples
    /// ```
    /// use tracery::grammar;
    /// # use tracery::Result;
    /// # fn main() -> Result<()> {
    /// let g = grammar! {
    ///     "start" => "#tool# is #description#!",
    ///     "tool" => "tracery",
    ///     "description" => [ "fun", "awesome" ]
    /// }?.with_default_rule("start");
    /// # let output = g.flatten(&mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_default_rule<S: Into<String>>(mut self, s: S) -> Grammar {
        self.set_default_rule(s);
        self
    }

    /// Sets a default rule
    ///
    /// # Examples
    /// ```
    /// use tracery::grammar;
    /// # use tracery::Result;
    /// # fn main() -> Result<()> {
    /// let mut g = grammar! {
    ///     "start" => "#tool# is #description#!",
    ///     "tool" => "tracery",
    ///     "description" => [ "fun", "awesome" ]
    /// }?;
    /// g.set_default_rule("start");
    /// # let output = g.flatten(&mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_default_rule<S: Into<String>>(&mut self, s: S) {
        self.default_rule = s.into();
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
    /// # Examples
    /// ```
    /// use tracery::grammar;
    /// # use tracery::Result;
    /// # fn main() -> Result<()> {
    /// let g = grammar! {
    ///     "origin" => "#tool# is #description#!",
    ///     "tool" => "tracery",
    ///     "description" => [ "fun", "awesome" ]
    /// }?;
    ///
    /// // Generate output (either "tracery is fun!" or "tracery is awesome!")
    /// let output = g.flatten(&mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
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
    /// If you wish to produce an output String without preserving changes, use
    /// [`flatten`].
    /// ```
    /// use tracery::grammar;
    /// # use tracery::Result;
    /// # fn main() -> Result<()> {
    /// let mut g = grammar! {
    ///     "origin" => "#tool# is #description#!",
    ///     "tool" => "tracery",
    ///     "description" => [ "fun", "awesome" ]
    /// }?;
    ///
    /// // Generate output (either "tracery is fun!" or "tracery is awesome!")
    /// let key = String::from("origin");
    /// let output = g.execute(&key, &mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Using a key created during a previous execution:
    ///
    /// ```
    /// use tracery::grammar;
    /// # use tracery::Result;
    /// # fn main() -> Result<()> {
    /// // This time, origin has a side-effect: it creates the rule 'aside'
    /// let mut g = grammar! {
    ///     "origin" => "#[aside:Rust is, too]tool# is #description#!",
    ///     "tool" => "tracery",
    ///     "description" => [ "fun", "awesome" ]
    /// }?;
    ///
    /// // Generate output (either "tracery is fun!" or "tracery is awesome!")
    /// let key = String::from("origin");
    /// let output = g.execute(&key, &mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    ///
    /// // The previous call to execute created the 'aside' rule
    /// let key = String::from("aside");
    /// // Generates the string "Rust is, too"
    /// let output = g.execute(&key, &mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "Rust is, too" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`flatten`]: struct.Grammar.html#method.flatten
    pub fn execute<R>(&mut self, key: &String, rng: &mut R) -> Result<String>
    where
        R: ?Sized + Rng,
    {
        let rule = match self.map.get(key) {
            Some(rules) => Ok(rules.last().unwrap().choose(rng).unwrap().clone()),
            None => Err(Error::MissingKeyError(key.clone())),
        }?;
        rule.execute(self, rng)
    }

    /// Creates a new Grammar from an input map of keys to rule lists
    ///
    /// # Examples
    /// ```
    /// # use tracery::Result;
    /// # use maplit::hashmap;
    /// # fn main() -> Result<()> {
    /// let map = hashmap! {
    ///     "origin" => vec![ "#tool# is #description#!" ],
    ///     "tool" => vec![ "tracery" ],
    ///     "description" => vec![ "fun", "awesome" ]
    /// };
    /// let g = tracery::from_map(map)?;
    /// # let output = g.flatten(&mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Any object implementing
    /// `IntoIterator<Item = (Into<String>, Into<Vec<Into<String>>>)>` will be
    /// accepted by this function, despite its name:
    ///
    /// ```
    /// # use tracery::Result;
    /// # fn main() -> Result<()> {
    /// let map = vec![ ("origin", vec![ "#tool# is #description#!" ]),
    ///                 ("tool", vec![ "tracery" ]),
    ///                 ("description", vec![ "fun", "awesome" ]) ];
    /// let g = tracery::from_map(map)?;
    /// # let output = g.flatten(&mut rand::thread_rng())?;
    /// # assert!(match output.as_str() {
    /// #     "tracery is fun!" | "tracery is awesome!" => true,
    /// #     _ => false,
    /// # });
    /// # Ok(())
    /// # }
    /// ```
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
            default_rule: ORIGIN.clone(),
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
    fn with_default_rule() -> Result<()> {
        let input = hashmap! {
            "a" => vec![ "a", "aa", "aaa" ]
        };
        let g = Grammar::from_map(input)?.with_default_rule("a");
        let res = g.flatten(&mut rand::thread_rng())?;
        assert_eq!(res.chars().next().unwrap(), 'a');

        Ok(())
    }

    #[test]
    fn set_default_rule() -> Result<()> {
        let input = hashmap! {
            "a" => vec![ "a", "aa", "aaa" ]
        };
        let mut g = Grammar::from_map(input)?;
        g.set_default_rule("a");
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
    fn execute() -> Result<()> {
        let input = hashmap! {
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
        let input = hashmap! {
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
        assert_eq!(
            "bazbar",
            grammar.execute(&String::from("origin"), &mut rand::thread_rng())?
        );
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
        assert!(matches!(
            grammar.execute(&origin, &mut rng),
            Err(Error::MissingKeyError(_))
        ));
        Ok(())
    }
}
