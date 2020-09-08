use inflector::string::pluralize;

use std::collections::BTreeMap;
use std::rc::Rc;

pub(crate) fn get_default_modifiers() -> BTreeMap<String, Rc<dyn Fn(&str) -> String>> {
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
        Rc::new(capitalize) as Rc<dyn Fn(&str) -> String>,
    );
    modifiers.insert(
        "capitalizeAll".into(),
        Rc::new(move |s: &str| {
            use split_preserve::SplitPreserveWS;
            SplitPreserveWS::new(s).map_words(capitalize).collect()
        }) as Rc<dyn Fn(&str) -> String>,
    );
    modifiers.insert(
        "inQuotes".into(),
        Rc::new(|s: &str| format!("\"{}\"", s)) as Rc<dyn Fn(&str) -> String>,
    );
    modifiers.insert(
        "comma".into(),
        Rc::new(|s: &str| {
            if s.ends_with(',') || s.ends_with('.') || s.ends_with('!') || s.ends_with('?') {
                s.to_string()
            } else {
                format!("{},", s)
            }
        }) as Rc<dyn Fn(&str) -> String>,
    );
    modifiers.insert(
        "s".into(),
        Rc::new(|s: &str| pluralize::to_plural(s)) as Rc<dyn Fn(&str) -> String>,
    );
    let is_vowel = |c: char| -> bool {
        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => true,
            _ => false,
        }
    };
    modifiers.insert(
        "a".into(),
        Rc::new(move |s: &str| {
            format!(
                "{} {}",
                match s.chars().next().map(is_vowel) {
                    Some(true) => "an",
                    _ => "a",
                },
                s
            )
        }) as Rc<dyn Fn(&str) -> String>,
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
        Rc::new(move |s: &str| {
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
        }) as Rc<dyn Fn(&str) -> String>,
    );
    modifiers
}

mod tests {
    #[test]
    fn capitalize() {
        let mods = super::get_default_modifiers();
        let c = &mods["capitalize"];
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
        let mods = super::get_default_modifiers();
        let c = &mods["capitalizeAll"];
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
        let mods = super::get_default_modifiers();
        let c = &mods["inQuotes"];
        assert_eq!(c(""), r#""""#);
        assert_eq!(c("hail eris"), r#""hail eris""#);
    }

    #[test]
    fn comma() {
        let mods = super::get_default_modifiers();
        let c = &mods["comma"];

        assert_eq!(c("a,"), "a,");
        assert_eq!(c("a."), "a.");
        assert_eq!(c("a!"), "a!");
        assert_eq!(c("a?"), "a?");

        assert_eq!(c("a"), "a,");
        assert_eq!(c(""), ",");
    }

    #[test]
    fn s() {
        let mods = super::get_default_modifiers();
        let c = &mods["s"];

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
        let mods = super::get_default_modifiers();
        let c = &mods["a"];

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
        let mods = super::get_default_modifiers();
        let c = &mods["ed"];

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
