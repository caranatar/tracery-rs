//! Rust port of `tracery`
//!
//! This library is a port of https://github.com/galaxykate/tracery, which implements Generative
//! grammars. Given a set of rules, written in a specific syntax, it will generate strings of text.
//!
//! Example:
//!
//! ```ignore
//! let source = r##"
//! {
//!     "origin": ["foo #bar#", "#baz# quux #quuux#"],
//!     "bar": ["bar", "BAR"],
//!     "baz": ["baz", "BaZ", "bAAZ"],
//!     "quuux": ["quick brown fox", "lazy dog"]
//! }
//! "##;
//!
//! let grammar = tracery::from_json(source).unwrap();
//! println!(grammar.flatten()) // => starting from the "origin" rule (which is selected by
//!                             //    default), fills in random
//!                             //    entries from the "bar", "baz", and "quuux" rules,
//!                             //    where called for in the "origin" text.
//! ```
//! or, even shorter:
//!
//! ```ignore
//! let source = r##"
//! {
//!     "origin": ["foo #bar#", "#baz# quux #quuux#"],
//!     "bar": ["bar", "BAR"],
//!     "baz": ["baz", "BaZ", "bAAZ"],
//!     "quuux": ["quick brown fox", "lazy dog"]
//! }"##;
//! tracery::flatten(source).unwrap();
//! ```
//!
//! So, in the example above, we might end up with `"foo bar"` or `"BaZ quux lazy dog"`, etc
//!
//! ## API
//!
//! In the example above, we used `Grammar.flatten`, but that is a convenience function that
//! does the following:
//!
//! ```ignore
//! let grammar = tracery::from_json(source).unwrap();
//! let flattened = grammar.flatten();
//! ```
//!
//! `.from_json` will parse the rule set out into a tree-like structure, and `.flatten` collapses that
//! tree-like structure into a single string.
//!
//! ## More `tracery` syntax
//!
//! Tracery allows for more than just word replacement. You can attach "actions" and "modifiers" to
//! rules as well. There are quite a few modifiers built-in to this library. Here is one:
//!
//! ```ignore
//! let source = r##"
//! {
//!     "origin": ["this word is in plural form: #noun.s#"],
//!     "noun": ["apple", "bear", "cat", "dog", "equine", "fish", "garbage"]
//! }"##;
//!
//! let grammar = tracery::from_json(source).unwrap();
//! println!(grammar.flatten());
//! ```
//!
//! This will generate sentences like:
//!
//! > "this word is in plural form: bears"
//!
//! or
//!
//! > "this word is in plural form: fishes"
//!
//! etc...
//!
//! Actions allow you to, for example, lock in a specific value for a `#tag#`, so that you can refer to it multiple
//! times in your story. Here is an example (modified from @galaxykate's official tutorial
//! http://www.crystalcodepalace.com/traceryTut.html)
//!
//! ```ignore
//! let source = r##"{
//!     "name": ["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"],
//!     "animal": ["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"],
//!     "mood": ["vexed","indignant","impassioned","wistful","astute","courteous"],
//!     "story": ["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."],
//!     "origin": ["#[hero:#name#][heroPet:#animal#]story#"]
//! }"##;
//! ```
//!
//! We see, in the "origin" rule, the use of actions to lock-in the value of `#hero#` and
//! `#heroPet#`, so that we can use those tags in the "story" rule, and know that the same
//! generated value will be used in all cases.

use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

mod grammar;
mod parser;
mod tag;

pub use crate::grammar::{Flatten, Grammar};

pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
    Grammar::from_json(s)
}

pub fn flatten<S: AsRef<str>>(s: S) -> Result<String> {
    from_json(s)?.flatten(&Grammar::new(), &mut BTreeMap::new())
}

#[derive(Debug, Clone)]
pub enum Error {
    ParseError(String),
    MissingKeyError(String),
    JsonError(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::JsonError(format!("{}", e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(ref s) => write!(f, "parse error: {}", s),
            Error::MissingKeyError(ref s) => write!(f, "missing key error: {}", s),
            Error::JsonError(ref s) => write!(f, "json error: {}", s),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError(ref s) => s,
            Error::MissingKeyError(ref s) => s,
            Error::JsonError(ref s) => s,
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::from_json;
    use crate::grammar::{Flatten, Grammar};
    use std::collections::BTreeMap;

    #[test]
    fn test_flatten() {
        let source = " { \"origin\": [\"foo #bar#\"], \"bar\": [\"bar\"] } ";
        assert_eq!(super::flatten(source).unwrap(), "foo bar".to_string());
    }

    #[test]
    fn test_with_actions() {
        let source = r##"{
                "name": ["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"],
                "animal": ["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"],
                "mood": ["vexed","indignant","impassioned","wistful","astute","courteous"],
                "story": ["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."],
                "origin": ["#[hero:#name#][heroPet:#animal#]story#"]
            }"##;
        match from_json(source) {
            Ok(g) => {
                g.flatten(&Grammar::new(), &mut BTreeMap::new()).unwrap();
            }
            Err(e) => println!("Error was {}", e),
        };
    }
}
