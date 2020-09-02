#![warn(missing_docs)]
//! Rust port of `tracery`
//!
//! This library is a port of https://github.com/galaxykate/tracery, which implements Generative
//! grammars. Given a set of rules, written in a specific syntax, it will generate strings of text.
//!
//! Example:
//!
//! ```
//! # use maplit::hashmap;
//! # use tracery::{from_map, Result};
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let source = hashmap! {
//!     "origin" => vec!["foo #bar#", "#baz# quux #qux#"],
//!     "bar" => vec!["bar", "BAR"],
//!     "baz" => vec!["baz", "BaZ", "bAZ"],
//!     "qux" => vec!["qux", "QUX"]
//! };
//!
//! let grammar = tracery::from_map(source).unwrap();
//! // Starting from the "origin" rule, which is selected by default, fills in
//! // random entries from the "bar", "baz", and "qux" rules, where called for
//! // in the "origin" text:
//! let flattened = grammar.flatten(&mut rand::thread_rng())?;
//! let matches = flattened.eq_ignore_ascii_case("foo bar") || flattened.eq_ignore_ascii_case("baz quux qux");
//! assert!(matches);
//! # Ok(())
//! # }
//! ```
//! or, even shorter:
//!
//! ```
//! # use tracery::Result;
//! # use maplit::hashmap;
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let source = hashmap! {
//!     "origin" => vec!["foo #bar#", "#baz# quux #qux#"],
//!     "bar" => vec!["bar", "BAR"],
//!     "baz" => vec!["baz", "BaZ", "bAZ"],
//!     "qux" => vec!["qux", "QUX"]
//! };
//! let flattened = tracery::flatten_map(source)?;
//! let matches = flattened.eq_ignore_ascii_case("foo bar") || flattened.eq_ignore_ascii_case("baz quux qux");
//! assert!(matches);
//! # Ok(())
//! # }
//! ```
//!
//! So, in the example above, we might end up with `"foo bar"` or `"BaZ quux lazy dog"`, etc
//!
//! ## API
//!
//! In the example above, we used `Grammar.flatten`, but that is a convenience function that
//! does the following:
//!
//! ```
//! # use tracery::{Grammar, Result};
//! # use maplit::hashmap;
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let grammar = tracery::from_map(hashmap! {
//!   "origin" => vec![ "#foo# is #bar#" ],
//!   "foo" => vec![ "tracery" ],
//!   "bar" => vec![ "fun" ]
//! }).unwrap();
//! let flattened = grammar.flatten(&mut rand::thread_rng())?;
//! assert_eq!(flattened, "tracery is fun");
//! # Ok(())
//! # }
//! ```
//!
//! `.from_map` will parse the rule set out into a tree-like structure, and
//! `.flatten` collapses that tree-like structure into a single string.
//!
//! ## More `tracery` syntax
//!
//! Tracery allows for more than just word replacement. You can attach "actions" and "modifiers" to
//! rules as well. There are quite a few modifiers built-in to this library. Here is one:
//!
//! ```
//! # use tracery::{Grammar, Result};
//! # use maplit::hashmap;
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let source = hashmap! {
//!     "origin" => vec!["this word is in plural form: #noun.s#"],
//!     "noun" => vec!["apple"]
//! };
//!
//! let grammar = tracery::from_map(source)?;
//! let flattened = grammar.flatten(&mut rand::thread_rng())?;
//! assert_eq!("this word is in plural form: apples", flattened);
//! # Ok(())
//! # }
//! ```
//!
//! Actions allow you to, for example, lock in a specific value for a `#tag#`, so that you can refer to it multiple
//! times in your story. Here is an example (modified from @galaxykate's official tutorial
//! http://www.crystalcodepalace.com/traceryTut.html)
//!
//! ```
//! # use tracery::Result;
//! # use maplit::hashmap;
//! # fn main() -> Result<()> {
//! let source = hashmap! {
//!     "name" => vec!["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"],
//!     "animal" => vec!["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"],
//!     "mood" => vec!["vexed","indignant","impassioned","wistful","astute","courteous"],
//!     "story" => vec!["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."],
//!     "origin" => vec!["#[hero:#name#][heroPet:#animal#]story#"]
//! };
//! println!("{}", tracery::flatten_map(source)?);
//! # Ok(())
//! # }
//! ```
//!
//! We see, in the "origin" rule, the use of actions to lock-in the value of `#hero#` and
//! `#heroPet#`, so that we can use those tags in the "story" rule, and know that the same
//! generated value will be used in all cases.

mod error;
pub use crate::error::Error;
mod execute;
pub(crate) use crate::execute::Execute;
mod grammar;
pub use crate::grammar::Grammar;
mod modifiers;
mod node;
use crate::node::Node;
mod parser;
mod rule;
use crate::rule::Rule;
mod tag;

/// Creates a new grammar from a JSON grammar string
#[cfg(feature = "tracery_json")]
pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
    use std::collections::HashMap;
    let map: HashMap<String, Vec<String>> = serde_json::from_str(s.as_ref())?;
    Grammar::from_map(map)
}

/// Creates a new grammar from an input map
pub fn from_map<I, K, C, S>(iter: I) -> Result<Grammar>
where
    I: IntoIterator<Item = (K, C)>,
    K: Into<String>,
    C: IntoIterator<Item = S>,
    S: Into<String>,
{
    Grammar::from_map(iter)
}

/// Creates a new grammar from a JSON grammar string, then uses it to create a
/// random output string
#[cfg(feature = "tracery_json")]
pub fn flatten_json<S: AsRef<str>>(s: S) -> Result<String> {
    from_json(s)?.flatten(&mut rand::thread_rng())
}

/// Creates a new grammar from an input map, then uses it to create a random
/// output string
pub fn flatten_map<I, K, C, S>(iter: I) -> Result<String>
where
    I: IntoIterator<Item = (K, C)>,
    K: Into<String>,
    C: IntoIterator<Item = S>,
    S: Into<String>,
{
    from_map(iter)?.flatten(&mut rand::thread_rng())
}

/// A convenience type for a `Result` of `T` or [`Error`]
///
/// [`Error`]: enum.Error.html
pub type Result<T> = ::std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[cfg(feature = "tracery_json")]
    use super::from_json;
    use super::from_map;
    use super::Result;
    use maplit::hashmap;

    #[test]
    fn test_flatten_map() {
        let source = hashmap!{
            "origin" => vec!["foo #bar#"],
            "bar" => vec!["bar"]
        };
        assert_eq!(super::flatten_map(source).unwrap(), "foo bar".to_string());
    }

    #[test]
    fn test_map_with_actions() -> Result<()> {
        let source = hashmap!{
            "name" => vec!["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"],
            "animal" => vec!["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"],
            "mood" => vec!["vexed","indignant","impassioned","wistful","astute","courteous"],
            "story" => vec!["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."],
            "origin" => vec!["#[hero:#name#][heroPet:#animal#]story#"]
        };
        let g = from_map(source)?;
        g.flatten(&mut rand::thread_rng())?;
        Ok(())
    }

    #[test]
    fn test_malformed_input() {
        let input = hashmap!{ "a" => vec!["#a"]};
        let res = from_map(input);
        assert!(matches!(res, Err(crate::Error::ParseError(_))));
    }

    #[test]
    #[cfg(feature = "tracery_json")]
    fn test_flatten_json() {
        let source = " { \"origin\": [\"foo #bar#\"], \"bar\": [\"bar\"] } ";
        assert_eq!(super::flatten_json(source).unwrap(), "foo bar".to_string());
    }

    #[test]
    #[cfg(feature = "tracery_json")]
    fn test_json_with_actions() -> Result<()> {
        let source = r##"{
                "name": ["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"],
                "animal": ["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"],
                "mood": ["vexed","indignant","impassioned","wistful","astute","courteous"],
                "story": ["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."],
                "origin": ["#[hero:#name#][heroPet:#animal#]story#"]
            }"##;
        let g = from_json(source)?;
        g.flatten(&mut rand::thread_rng())?;
        Ok(())
    }

    #[test]
    #[cfg(feature = "tracery_json")]
    fn malformed_json() {
        let input = r#"{ "a": ["a"],}"#;
        let res = from_json(input);
        assert!(matches!(res, Err(crate::Error::JsonError(_))));
    }
}
