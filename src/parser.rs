use std::collections::BTreeMap;

use pest_derive::Parser;
use pest::Parser;

use crate::Error;
use crate::grammar::Rule as TRule;
use crate::tag::Tag;

#[derive(Parser)]
#[grammar = "tracery.pest"]
struct TraceryParser;

type PestError = pest::error::Error<Rule>;

fn _parse_str<S: AsRef<str>>(s: S) -> Result<TRule, PestError> {
    let parsed_str = TraceryParser::parse(Rule::rule, s.as_ref())?.next().unwrap();

    for part in parsed_str.into_inner() {
        println!("part: {:?}", part);
    }
    unimplemented!()
}

pub fn parse_str<S: AsRef<str>>(s: S) -> Result<TRule, Error> {
    _parse_str(s).map_err(|e| {
        Error::ParseError(format!("{}", e))
    })
}

fn parse_action(_s: pest::iterators::Pair<Rule>) -> Result<(String, Rule), PestError> {
    unimplemented!()
}

fn _parse_tag<S: AsRef<str>>(s: S) -> Result<Tag, PestError> {
    let parsed_tag = TraceryParser::parse(Rule::tag, s.as_ref())?.next().unwrap();

    let actions = BTreeMap::new();
    let mut tagname = "";
    let mut modifiers = Vec::new();
    for part in parsed_tag.into_inner() {
        match part.as_rule() {
            Rule::action => {
                let _ = parse_action(part);
            },
            Rule::tagname => {
                tagname = part.as_str();
            },
            Rule::modifier => {
                let modifier = part.into_inner().next().unwrap().as_str();
                modifiers.push(modifier);
            },
            _ => unreachable!(),
        }
    }

    Ok(Tag::new(tagname).with_actions(actions).with_modifiers(modifiers))
}

pub fn parse_tag<S: AsRef<str>>(s: S) -> Result<Tag, Error> {
    _parse_tag(s).map_err(|e| {
        Error::ParseError(format!("{}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_tagname() -> Result<(), Error> {
        let tag = parse_tag("#one#")?;
        assert_eq!(tag.key, "one");
        Ok(())
    }

    #[test]
    fn parse_tag_with_tag_action() {
        let _ = parse_tag("#[one:#two#]tagname#");
    }

    #[test]
    fn parse_tag_with_text_action() {
        let _ = parse_tag("#[one:a:b.c d]tagname#");
    }

    #[test]
    fn parse_tag_with_modifiers() -> Result<(), Error> {
        let tag = parse_tag("#one.two.three#")?;
        assert_eq!(tag.key, "one");
        assert_eq!(tag.modifiers, vec!["two", "three"]);
        Ok(())
    }

    #[test]
    fn parse_tag_complicated() {
        let _ = parse_tag("#[e:#[a:#b.c#]d#][f:#g.h#]i.j.k#");
    }

    #[test]
    fn parse_rule() {
        let _ = parse_str("hello. [a][b]: #name#");
    }
}
