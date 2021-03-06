use pest::Parser;
use pest_derive::Parser;

use crate::tag::Tag;
use crate::Error;
use crate::Node;
use crate::Rule as TRule;

#[derive(Parser)]
#[grammar = "tracery.pest"]
struct TraceryParser;

type PestError = pest::error::Error<Rule>;

fn parse_rule<S: AsRef<str>>(s: S) -> Result<TRule, PestError> {
    let parsed_str = TraceryParser::parse(Rule::rule, s.as_ref())?
        .next()
        .unwrap();

    let nodes = parsed_str.into_inner().try_fold(Vec::new(), |mut acc, p| {
        match p.as_rule() {
            Rule::text => acc.push(Node::Text(p.as_str().to_string())),
            Rule::tag => acc.push(Node::Tag(parse_tag_pair(p)?)),
            Rule::actions => acc.push(Node::Tag(parse_actions(p)?)),
            _ => unreachable!(),
        }
        Ok(acc)
    })?;

    Ok(TRule::new(nodes))
}

pub(crate) fn parse_str<S: AsRef<str>>(s: S) -> Result<TRule, Error> {
    parse_rule(s).map_err(|e| Error::ParseError(format!("{}", e)))
}

fn parse_actions(a: pest::iterators::Pair<Rule>) -> Result<Tag, PestError> {
    let actions = a.into_inner().try_fold(Vec::new(), |mut acc, p| {
        match p.as_rule() {
            Rule::action => {
                let (label, rule) = parse_action(p)?;
                acc.push((label, rule));
            }
            _ => unreachable!(),
        }
        Ok(acc)
    })?;
    Ok(Tag::empty().with_actions(actions))
}

fn parse_action(a: pest::iterators::Pair<Rule>) -> Result<(Option<String>, TRule), PestError> {
    let mut tagname = None;
    let mut rule = None;
    for part in a.into_inner() {
        match part.as_rule() {
            Rule::tagname => {
                tagname = Some(part.as_str().to_string());
            }
            // action_rhs for labeled actions; tag for unlabeled actions
            Rule::action_rhs | Rule::tag => {
                rule = Some(parse_rule(part.as_str())?);
            }
            _ => unreachable!(),
        }
    }

    Ok((tagname, rule.unwrap()))
}

fn parse_tag_pair(s: pest::iterators::Pair<Rule>) -> Result<Tag, PestError> {
    let mut actions = Vec::new();
    let mut tagname = "";
    let mut modifiers = Vec::new();
    for part in s.into_inner() {
        match part.as_rule() {
            Rule::action => {
                let (key, action) = parse_action(part)?;
                actions.push((key, action));
            }
            Rule::tagname => {
                tagname = part.as_str();
            }
            Rule::modifier => {
                let modifier = part.into_inner().next().unwrap().as_str();
                modifiers.push(modifier);
            }
            _ => unreachable!(),
        }
    }

    Ok(Tag::new(tagname)
        .with_actions(actions)
        .with_modifiers(modifiers))
}

#[cfg(test)]
pub(crate) fn parse_tag<S: AsRef<str>>(s: S) -> Result<Tag, Error> {
    let tag_pair = TraceryParser::parse(Rule::tag, s.as_ref())
        .map_err(|e| Error::ParseError(format!("{}", e)))?
        .next()
        .unwrap();
    parse_tag_pair(tag_pair).map_err(|e| Error::ParseError(format!("{}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tagname() -> Result<(), Error> {
        let tag = parse_tag("#one#")?;
        assert_eq!(tag.key.unwrap(), "one");
        Ok(())
    }

    #[test]
    fn parse_text() -> Result<(), Error> {
        let src = "this is some text";
        let rule = parse_str(src)?;
        assert_eq!(rule.0, vec![Node::Text(src.to_string())]);
        Ok(())
    }

    #[test]
    fn parse_tag_with_tag_action() -> Result<(), Error> {
        let tag = parse_tag("#[one:#two#]tagname#")?;
        assert_eq!(tag.key.unwrap(), "tagname");
        assert_eq!(tag.actions.len(), 1);
        let action = &tag.actions[0];
        assert_eq!(action.label, Some(String::from("one")));
        assert_eq!((action.rule).0, vec![Node::Tag(Tag::new("two"))]);
        Ok(())
    }

    #[test]
    fn parse_tag_with_text_action() -> Result<(), Error> {
        let tag = parse_tag("#[one:a:b.c d]tagname#")?;
        assert_eq!(tag.key.unwrap(), "tagname");
        assert_eq!(tag.actions.len(), 1);
        let action = &tag.actions[0];
        assert_eq!(action.label, Some(String::from("one")));
        assert_eq!((action.rule).0, vec![Node::Text("a:b.c d".to_string())]);
        Ok(())
    }

    #[test]
    fn parse_tag_with_modifiers() -> Result<(), Error> {
        let tag = parse_tag("#one.two.three#")?;
        assert_eq!(tag.key.unwrap(), "one");
        assert_eq!(tag.modifiers, vec!["two", "three"]);
        Ok(())
    }

    #[test]
    fn parse_tag_complicated() -> Result<(), Error> {
        let tag = parse_tag("#[e:#[a:#b.c#]d#][f:#g.h#]i.j.k#")?;
        assert_eq!(tag.key.unwrap(), "i");
        assert_eq!(tag.modifiers, vec!["j", "k"]);
        Ok(())
    }

    #[test]
    fn parse_mixed_rule() -> Result<(), Error> {
        let rule = parse_str("hello. [a][b]: #name# more after")?;

        assert_eq!(
            rule.0,
            vec![
                Node::Text("hello. [a][b]: ".to_string()),
                Node::Tag(Tag::new("name")),
                Node::Text(" more after".to_string())
            ]
        );

        Ok(())
    }

    #[test]
    fn parse_rule_with_hash_dot() -> Result<(), Error> {
        let src = "#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the \
                   #heroPet# was always too #mood#.";
        let rule = parse_str(src)?;
        assert_eq!(
            rule.0,
            vec![
                Node::Tag(Tag::new("hero")),
                Node::Text(" traveled with her pet ".into()),
                Node::Tag(Tag::new("heroPet")),
                Node::Text(".  ".into()),
                Node::Tag(Tag::new("hero")),
                Node::Text(" was never ".into()),
                Node::Tag(Tag::new("mood")),
                Node::Text(", for the ".into()),
                Node::Tag(Tag::new("heroPet")),
                Node::Text(" was always too ".into()),
                Node::Tag(Tag::new("mood")),
                Node::Text(".".into()),
            ]
        );

        Ok(())
    }

    #[test]
    fn parse_tag_multi_action() -> Result<(), Error> {
        let src = "#[one:#two#][three:#four#]tagname.s.capitalize#";
        let mut actions = Vec::new();
        actions.push((Some("one".to_string()), parse_str("#two#").unwrap()));
        actions.push((Some("three".to_string()), parse_str("#four#").unwrap()));
        let tag = parse_tag(src)?;
        assert_eq!(
            tag,
            Tag::new("tagname")
                .with_actions(actions)
                .with_modifiers(vec!["s", "capitalize"])
        );
        Ok(())
    }
}
