#![deny(missing_docs)]
//! Rust implementation of the tracery generative grammar language.
//!
//! This library is a Rust port/implementation of [`tracery`], the generative
//! grammar language designed and created by [`Kate Compton`]. Given a set of
//! rules written in the tracery syntax, it will use them to procedurally
//! generate strings of text. For more information about the tracery language,
//! see the [`Language Concepts`] section below.
//!
//! # Usage
//! Usage of the library can be divided into two areas: creation of grammars and
//! the generation of output strings.
//!
//! ## Grammar Creation
//! Grammars can be created using the [`grammar!`] macro, from an any iterable
//! rust object of strings and associated lists of strings, or for compatibility
//! with the original tracery, from a string representing a JSON map.
//!
//! ### The grammar! macro
//! Accepts input in the form `"key" => [ "list", "of", "rules" ]` or, in the
//! case of a key having only one rule, `"key" => "rule"`. Equivalent to
//! manually building a map and then calling [`Grammar::from_map`]
//!
//! ```
//! use tracery::grammar;
//! # use tracery::Result;
//! # fn main() -> Result<()> {
//! let g = grammar! {
//!     "origin" => "#tool# is #description#!",
//!     "tool" => "tracery",
//!     "description" => [ "fun", "awesome" ]
//! }?;
//! # let output = g.flatten(&mut rand::thread_rng())?;
//! # assert!(match output.as_str() {
//! #     "tracery is fun!" | "tracery is awesome!" => true,
//! #     _ => false,
//! # });
//! # Ok(())
//! # }
//! ```
//!
//! ### From a map/iterator
//! A grammar can be created from any object implementing, essentially,
//! `IntoIterator<Item = (Into<String>, Into<Vec<Into<String>>)>`. For example,
//! `HashMap<String, Vec<String>>` or `BTreeMap<&str, &[&str]>`.
//!
//! ```
//! # use tracery::Result;
//! # use maplit::hashmap;
//! # fn main() -> Result<()> {
//! let map = hashmap! {
//!     "origin" => vec![ "#tool# is #description#!" ],
//!     "tool" => vec![ "tracery" ],
//!     "description" => vec![ "fun", "awesome" ]
//! };
//! let g = tracery::from_map(map)?;
//! # let output = g.flatten(&mut rand::thread_rng())?;
//! # assert!(match output.as_str() {
//! #     "tracery is fun!" | "tracery is awesome!" => true,
//! #     _ => false,
//! # });
//! # Ok(())
//! # }
//! ```
//!
//! ### From a JSON string
//! For compatibility with the original tracery, a Grammar can be created from a
//! string representing a JSON object. This feature is controlled by the
//! `tracery_json` feature, which is enabled by default. It can be turned off if
//! you do not require this functionality.
//!
#![cfg_attr(feature = "tracery_json", doc = r#"
```
"#)]
#![cfg_attr(not(feature = "tracery_json"), doc = r#"
```ignore
"#)]
//! # use tracery::Result;
//! # use maplit::hashmap;
//! # fn main() -> Result<()> {
//! let json = r##"{
//!     "origin": [ "#tool# is #description#!" ],
//!     "tool": [ "tracery" ],
//!     "description": [ "fun", "awesome" ]
//! }"##;
//! let g = tracery::from_json(json)?;
//! # let output = g.flatten(&mut rand::thread_rng())?;
//! # assert!(match output.as_str() {
//! #     "tracery is fun!" | "tracery is awesome!" => true,
//! #     _ => false,
//! # });
//! # Ok(())
//! # }
//! ```
//!
//! # Language Concepts
//! A *grammar* is a map from a set of string *key*s to a stack of *rulesets*,
//! notionally rooted at an "origin" node, associated by default with the key
//! "origin"
//!
//! A *key* is any valid UTF-8 String that does not contain the reserved
//! characters `[`, `]`, `.`, `:`, or `#`. A key is associated with a stack of
//! rulesets, and the topmost ruleset is used when expanding the key or popping
//! a ruleset off the stack using a pop action.
//!
//! A *ruleset* is a list (internally a `Vec<String>`) of strings, each
//! representing a possible expansion of the associated key, to be chosen at
//! random when expanding that key, containing one or more *plaintexts*, *tags*,
//! or *action*s.
//!
//! An *action* is enclosed by square brackets (`[`, `]`) and can be either
//! *labeled* or *unlabeled*.
//!
//! A *labeled action* takes the form `[key:rule]`, where `rule` is any valid
//! tracery rule string, and `key` is a valid key. The rule will be executed and
//! the result pushed onto the top of the ruleset stack associated with `key`.
//! If `key` does not exist already, it will be created. A special exception is
//! a pop action which takes the form `[key:POP]`, and will pop the top ruleset
//! off the stack of rulesets associated with `key`. If the stack becomes empty,
//! the key will be deleted from the associated grammar.
//!
//! An *unlabeled action* is a single *tag* encased in square brackets such as
//! `[#setPronouns#]` and is typically used to call a function-like ruleset
//! which will use labeled actions to set values for some set of keys (like
//! setting a character's pronouns for a story).
//!
//! A *tag* is a key, encased in hashes (`#`), which will be replaced in the
//! output by a random rule chosen from the topmost ruleset of the key's
//! associated ruleset stack. The key in a tag can also be preceded by one or
//! more actions, which will be executed before the tag is expanded. Some
//! examples of valid tags include: `#foo#`, `#[foo:#bar#]baz#`, and
//! `#[#setPronouns#][#setJob#][#setPet#]hero#`.
//!
//! A *plaintext* is any text in a rule which is not a tag or action.

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

#[doc(hidden)]
#[macro_export]
macro_rules! grammar_item {
    ($map:ident, ) => {};
    ($map:ident, $key:literal => [$($value: literal),+ $(,)?] $(, $($rest: tt)*)?) => {
        $map.insert($key, vec!($($value,)+));
        $($crate::grammar_item!($map, $($rest)*))?
    };
    ($map:ident, $key:literal => $value: literal $(, $($rest: tt)*)?) => {
        $map.insert($key, vec!($value));
        $($crate::grammar_item!($map, $($rest)*))?
    };
}

/// Convenience macro that allows for shorthand creation of [`Grammar`]s.
///
/// Accepts input in the form `"key" => [ "list", "of", "rules" ]` or, in the
/// case of a key having only one rule, `"key" => "rule"`. Equivalent to
/// manually building a map and then calling [`Grammar::from_map`]
///
/// # Returns
/// Result<[`Grammar`], [`Error`]>
///
/// # Example
///
/// ```
/// # use tracery::{grammar, Result};
/// # fn main() -> Result<()> {
/// // Declare the grammar
/// let g = grammar! {
///     "origin" => "#tool# is #description#!",
///     "tool" => "tracery",
///     "description" => [ "fun", "awesome" ]
/// }?;
///
/// // Randomly produce the string "tracery is fun!" or "tracery is awesome!"
/// # let output =
/// g.flatten(&mut rand::thread_rng())?;
///
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
/// [`Grammar::from_map`]: struct.Grammar.html#method.from_map
/// [`Result`]: type.Result.html
#[macro_export]
macro_rules! grammar {
    ($($input: tt)+) => {
        {
            let mut _map = std::collections::HashMap::new();
            $crate::grammar_item!(_map, $($input)+);
            $crate::from_map(_map)
        }
    }
}

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
    from_json(s)?.execute(&crate::grammar::ORIGIN, &mut rand::thread_rng())
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
    from_map(iter)?.execute(&crate::grammar::ORIGIN, &mut rand::thread_rng())
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
    use super::grammar;
    use super::Result;
    use maplit::hashmap;

    #[test]
    fn test_macro() -> Result<()> {
        let g = grammar! {
            "origin" => "#foo#",
            "foo" => ["a", "aa"]
        }?;
        let res = g.flatten(&mut rand::thread_rng())?;
        assert_eq!(res.chars().next().unwrap(), 'a');
        Ok(())
    }

    #[test]
    fn test_flatten_map() {
        let source = hashmap! {
            "origin" => vec!["foo #bar#"],
            "bar" => vec!["bar"]
        };
        assert_eq!(super::flatten_map(source).unwrap(), "foo bar".to_string());
    }

    #[test]
    fn test_map_with_actions() -> Result<()> {
        let source = hashmap! {
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
        let input = hashmap! { "a" => vec!["#a"]};
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
