use inflector::string::pluralize;
use rand::seq::SliceRandom;
use std::collections::BTreeMap;
use std::default::Default;

use super::{Error, Result};
use crate::parser::parse_str;
use crate::Flatten;
use crate::Rule;

/// Represents a single grammar
///
/// This is the main data type used with this library.
pub struct Grammar {
    map: BTreeMap<String, Vec<Rule>>,
    default_rule: String,
    modifier_registry: BTreeMap<String, Box<dyn Fn(&str) -> String>>,
}

impl Default for Grammar {
    fn default() -> Grammar {
        let mut modifiers = BTreeMap::new();
        let capitalize = |s: &str| {
            let mut iter = s.chars();
            let u = iter.next().map(|c| c.to_uppercase().to_string());
            format!(
                "{}{}",
                u.unwrap_or_else(String::default),
                iter.collect::<String>()
            )
        };
        modifiers.insert(
            "capitalize".into(),
            Box::new(capitalize) as Box<dyn Fn(&str) -> String>,
        );
        modifiers.insert(
            "capitalizeAll".into(),
            Box::new(move |s: &str| {
                use split_preserve::SplitPreserveWS;
                SplitPreserveWS::new(s).map_words(capitalize).collect()
            }) as Box<dyn Fn(&str) -> String>,
        );
        modifiers.insert(
            "inQuotes".into(),
            Box::new(|s: &str| format!("\"{}\"", s)) as Box<dyn Fn(&str) -> String>,
        );
        modifiers.insert(
            "comma".into(),
            Box::new(|s: &str| {
                if s.ends_with(',') || s.ends_with('.') || s.ends_with('!') || s.ends_with('?') {
                    s.to_string()
                } else {
                    format!("{},", s)
                }
            }) as Box<dyn Fn(&str) -> String>,
        );
        modifiers.insert(
            "s".into(),
            Box::new(|s: &str| pluralize::to_plural(s)) as Box<dyn Fn(&str) -> String>,
        );
        let is_vowel = |c: char| -> bool {
            match c {
                'a' | 'e' | 'i' | 'o' | 'u' => true,
                _ => false,
            }
        };
        modifiers.insert(
            "a".into(),
            Box::new(move |s: &str| {
                format!(
                    "{} {}",
                    match s.chars().next().map(is_vowel) {
                        Some(true) => "an",
                        _ => "a",
                    },
                    s
                )
            }) as Box<dyn Fn(&str) -> String>,
        );

        // Gets a char offset -n from the end. Returns None if n is larger than
        // len, returns s.get(s.len()-n) otherwise
        let get_neg = |s: &str, n: usize| -> Option<char> {
            if n > s.len() {
                None
            } else {
                s.chars().nth(s.len() - n)
            }
        };
        modifiers.insert(
            "ed".into(),
            Box::new(move |s: &str| {
                use split_preserve::{SplitPreserveWS, Token};
                // Split, preserving whitespace
                let mut iter = SplitPreserveWS::new(s);

                // Consume and save any leading whitespace as `prefix`
                let mut first = iter.next();
                let mut prefix: Vec<String> = Vec::new();
                while let Some(Token::Whitespace(s)) = first {
                    prefix.push(s.to_string());
                    first = iter.next();
                }
                let prefix: String = prefix.join("");

                // Process the first word
                let first = first
                    .and_then(|t| match t {
                        Token::Other(s) => Some(s),
                        _ => None,
                    })
                    .map(|s| match get_neg(s, 1) {
                        Some('y') => match get_neg(s, 2).map(is_vowel) {
                            Some(true) => format!("{}{}", s, "ed"),
                            _ => format!("{}{}", &s[..s.len() - 1], "ied"),
                        },
                        Some('e') => format!("{}{}", s, "d"),
                        Some(_) | None => format!("{}{}", s, "ed"),
                    })
                    .unwrap_or_else(String::default);

                // Collect the rest as a string
                let rest: String = iter
                    .map(|t| match t {
                        Token::Other(s) => s.to_string(),
                        Token::Whitespace(s) => s.to_string(),
                    })
                    .collect();

                // Stitch prefix, first, and rest together into one String
                format!("{}{}{}", prefix, first, rest,)
            }) as Box<dyn Fn(&str) -> String>,
        );
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
    pub fn get_rule(&self, key: &str) -> Option<&Vec<Rule>> {
        self.map.get(key)
    }

    /// Creates a new grammar from a JSON grammar string
    pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
        let source: BTreeMap<String, Vec<String>> = serde_json::from_str(s.as_ref())?;
        let mut me = Grammar::new();
        for (key, value) in source.into_iter() {
            let rules: Vec<Rule> = value.iter().map(parse_str).collect::<Result<Vec<_>>>()?;
            me.add_rules(key, rules)?;
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

    /// Adds a rule with the given key and list of rules to the `Grammar`
    ///
    /// # Returns
    /// `Ok(())` on success
    /// [`Error`] otherwise
    ///
    /// [`Error`]: enum.Error.html
    pub fn add_rules<S: Into<String>>(&mut self, name: S, rules: Vec<Rule>) -> Result<()> {
        self.map.insert(name.into(), rules);
        Ok(())
    }
}

impl Flatten for Grammar {
    fn flatten(&self, _: &Grammar, overrides: &mut BTreeMap<String, String>) -> Result<String> {
        if !self.map.contains_key(&self.default_rule) {
            return Err(Error::MissingKeyError(format!(
                "Grammar does not contain key {}",
                self.default_rule
            )));
        }

        match self.map.get(&self.default_rule) {
            Some(rules) => {
                let mut rng = rand::thread_rng();
                let rule = rules.choose(&mut rng).unwrap();
                rule.flatten(&self, overrides)
            }
            None => Ok("".to_string()),
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
        let res = g.flatten(&g, &mut BTreeMap::new());
        assert!(matches!(res, Err(Error::MissingKeyError(_))));

        Ok(())
    }

    #[test]
    fn set_default_rule() -> Result<()> {
        let input = r#"{
            "a": ["a", "aa", "aaa"]
        }"#;
        let g = Grammar::from_json(input)?.default_rule("a");
        let res = g.flatten(&g, &mut BTreeMap::new())?;
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
    }
}
