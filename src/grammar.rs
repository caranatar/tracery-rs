use inflector::cases::titlecase;
use inflector::string::pluralize;
use rand::seq::SliceRandom;
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt;

use super::{Error, Result};
use crate::parser::parse_str;
use crate::tag::Tag;

/// Represents a single grammar
///
/// This is the main data type used with this library.
pub struct Grammar {
    map: BTreeMap<String, Vec<Rule>>,
    default_rule: String,
    modifier_registry: BTreeMap<String, Box<dyn Fn(&str) -> String>>,
}

impl fmt::Debug for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Grammar {{ map: {:?}, default_rule: {:?} }}",
            self.map, self.default_rule
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule(pub(crate) Vec<Node>);

impl Rule {
    pub fn new(nodes: Vec<Node>) -> Rule {
        Rule(nodes)
    }

    pub fn parse<S: AsRef<str>>(source: S) -> Result<Rule> {
        parse_str(source.as_ref())
    }
}

/// Represents a part of a single expandable string
///
/// This is used to represent both the plain text, and the expandable text sections of a string.
///
/// # Example
///
/// ```ignore
/// let nodes = vec![
///     Node::Tag(Tag::new("one")),
///     Node::Text(" is the loneliest number".into()),
/// ];
///
/// assert_eq!(parser::parse_str("#one# is the loneliest number").unwrap(), nodes);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Tag(Tag),
    Text(String),
}

impl From<Tag> for Node {
    fn from(tag: Tag) -> Node {
        Node::Tag(tag)
    }
}

impl From<String> for Node {
    fn from(s: String) -> Node {
        Node::Text(s)
    }
}

impl Node {
    pub fn tag(s: &str) -> Result<Node> {
        Ok(Node::Tag(Tag::parse(s)?))
    }

    pub fn text(s: &str) -> Node {
        Node::Text(s.to_string())
    }
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
            Box::new(|s: &str| titlecase::to_title_case(s)) as Box<dyn Fn(&str) -> String>,
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
                let mut iter = s.splitn(2, char::is_whitespace);
                let first = iter.next().map(|s| match get_neg(s, 1) {
                    Some('y') => match get_neg(s, 2).map(is_vowel) {
                        Some(true) => format!("{}{}", s, "ed"),
                        _ => format!("{}{}", &s[..s.len() - 1], "ied"),
                    },
                    Some('e') => format!("{}{}", s, "d"),
                    Some(_) => format!("{}{}", s, "ed"),
                    None => s.to_string(),
                }).unwrap_or_else(String::default);
                let rest = iter.next().unwrap_or_else(|| "");
                format!(
                    "{} {}",
                    first,
                    rest,
                )
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
    pub fn new() -> Grammar {
        Default::default()
    }

    pub fn get_modifier(&self, modifier: &str) -> Option<&dyn Fn(&str) -> String> {
        self.modifier_registry.get(modifier).map(|x| x.as_ref())
    }

    pub fn get_rule(&self, key: &str) -> Option<&Vec<Rule>> {
        self.map.get(key)
    }

    pub fn from_json<S: AsRef<str>>(s: S) -> Result<Grammar> {
        let source: BTreeMap<String, Vec<String>> = serde_json::from_str(s.as_ref())?;
        let mut me = Grammar::new();
        for (key, value) in source.into_iter() {
            let rules: Vec<Rule> = value.iter().map(parse_str).collect::<Result<Vec<_>>>()?;
            me.add_rules(key, rules)?;
        }
        Ok(me)
    }

    pub fn default_rule<S: Into<String>>(mut self, s: S) -> Grammar {
        self.default_rule = s.into();
        self
    }

    pub fn add_rules<S: Into<String>>(&mut self, name: S, rules: Vec<Rule>) -> Result<()> {
        self.map.insert(name.into(), rules);
        Ok(())
    }
}

pub trait Flatten {
    fn flatten(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
    ) -> Result<String>;
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

impl Flatten for Rule {
    fn flatten(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
    ) -> Result<String> {
        let parts = self
            .0
            .iter()
            .map(|n| n.flatten(grammar, overrides))
            .collect::<Result<Vec<String>>>()?;
        Ok(parts.join(""))
    }
}

impl Flatten for Node {
    fn flatten(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
    ) -> Result<String> {
        match self {
            Node::Tag(ref tag) => tag.flatten(grammar, overrides),
            Node::Text(ref s) => s.flatten(grammar, overrides),
        }
    }
}

impl Flatten for String {
    fn flatten(&self, _: &Grammar, _: &mut BTreeMap<String, String>) -> Result<String> {
        Ok(self.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
