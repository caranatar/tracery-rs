#![warn(missing_docs)]
//! Rust port of `tracery`
//!
//! This library is a port of https://github.com/galaxykate/tracery, which implements Generative
//! grammars. Given a set of rules, written in a specific syntax, it will generate strings of text.
//!
//! Example:
//!
//! ```
//! # use tracery::{from_json, Result};
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let source = r##"
//! {
//!     "origin": ["foo #bar#", "#baz# quux #qux#"],
//!     "bar": ["bar", "BAR"],
//!     "baz": ["baz", "BaZ", "bAZ"],
//!     "qux": ["qux", "QUX"]
//! }
//! "##;
//!
//! let grammar = tracery::from_json(source).unwrap();
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
//! # use tracery::{flatten, Result};
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let source = r##"
//! {
//!     "origin": ["foo #bar#", "#baz# quux #qux#"],
//!     "bar": ["bar", "BAR"],
//!     "baz": ["baz", "BaZ", "bAZ"],
//!     "qux": ["qux", "QUX"]
//! }
//! "##;
//! let flattened = tracery::flatten(source)?;
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
//! # use tracery::{from_json, Grammar, Result};
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let grammar = tracery::from_json(r##"{
//!   "origin": [ "#foo# is #bar#" ],
//!   "foo": [ "tracery" ],
//!   "bar": [ "fun" ]
//! }"##)?;
//! let flattened = grammar.flatten(&mut rand::thread_rng())?;
//! assert_eq!(flattened, "tracery is fun");
//! # Ok(())
//! # }
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
//! ```
//! # use tracery::{from_json, Grammar, Result};
//! # use std::collections::BTreeMap;
//! # fn main() -> Result<()> {
//! let source = r##"
//! {
//!     "origin": ["this word is in plural form: #noun.s#"],
//!     "noun": ["apple"]
//! }"##;
//!
//! let grammar = tracery::from_json(source)?;
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
//! # use tracery::{flatten, Result};
//! # fn main() -> Result<()> {
//! let source = r##"{
//!     "name": ["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"],
//!     "animal": ["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"],
//!     "mood": ["vexed","indignant","impassioned","wistful","astute","courteous"],
//!     "story": ["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."],
//!     "origin": ["#[hero:#name#][heroPet:#animal#]story#"]
//! }"##;
//! println!("{}", tracery::flatten(source)?);
//! # Ok(())
//! # }
//! ```
//!
//! We see, in the "origin" rule, the use of actions to lock-in the value of `#hero#` and
//! `#heroPet#`, so that we can use those tags in the "story" rule, and know that the same
//! generated value will be used in all cases.

mod error;
pub use crate::error::Error;
mod flatten;
pub use crate::flatten::Flatten;
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
pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
    Grammar::from_json(s)
}

/// Creates a new grammar from a JSON grammar string, then uses it to create a
/// random output string
pub fn flatten<S: AsRef<str>>(s: S) -> Result<String> {
    from_json(s)?.flatten(&mut rand::thread_rng())
}

/// A convenience type for a `Result` of `T` or [`Error`]
///
/// [`Error`]: enum.Error.html
pub type Result<T> = ::std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::from_json;

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
                g.flatten(&mut rand::thread_rng()).unwrap();
            }
            Err(e) => println!("Error was {}", e),
        };
    }

    #[test]
    fn malformed_json() {
        let input = r#"{ "a": ["a"],}"#;
        let res = from_json(input);
        assert!(matches!(res, Err(crate::Error::JsonError(_))));
    }
}
