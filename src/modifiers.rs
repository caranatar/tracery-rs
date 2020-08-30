use inflector::string::pluralize;

use std::collections::BTreeMap;

pub(crate) fn get_default_modifiers() -> BTreeMap<String, Box<dyn Fn(&str) -> String>> {
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
    modifiers
}
