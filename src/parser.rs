use nom::IResult;
use nom::Err as NErr;
use nom::{ named, delimited, do_parse, tag, map, map_res, take_until, alt };
use std::string::ToString;

use std::collections::BTreeMap;
use grammar::{Rule, Node};
use super::{Result, Error};
use tag::Tag;

pub fn rest_s(i: &str) -> IResult<&str, &str> {
    IResult::Ok(("", i))
}

named!(action<&str, (String, Rule)>,
       delimited!(
           tag!("["),
           do_parse!(
               label: map!(take_until!(":"), ToString::to_string) >>
               tag!(":") >>
               content: map_res!(take_until!("]"), Rule::parse) >>
               (label, content)
            ),
           tag!("]")
        )
);

named!(modifier<&str, &str>, do_parse!(
            tag!(".") >>
            label: alt!(take_until!(".") | take_until!("#")) >>
            (label)
));

named!(pub _parse_tag<&str, Tag>,
        delimited!(
            tag!("#"),
            do_parse!(
                actions: many0!(action) >>
                key: is_not!(".#") >>
                modifiers: many0!(modifier) >>
                (Tag::new(key).with_actions(actions.into_iter().collect::<BTreeMap<_, _>>()).with_modifiers(modifiers))
            ),
            tag!("#")
        )
);

named!(pub tag_as_node<&str, Node>,
       map!(
            _parse_tag,
            From::from
       )
);

named!(pub text_as_node<&str, Node>,
       map!(
                alt!(take_until!("#") | rest_s),
            Node::text
       )
);
named!(pub tag_or_text<&str, Node>, alt!(tag_as_node | text_as_node));
//named!(_parse_str<&str, Vec<Node> >, many0!(tag_or_text));

fn _parse_str(s: &str) -> nom::IResult<&str, Vec<Node>> {
    let mut nodes = Vec::new();
    let mut input = s;
    loop {
        let (r, n) = tag_or_text(input)?;
        nodes.push(n);
        input = r;
        if input.len() == 0 {
            break;
        }
    }
    Ok(("", nodes))
}

pub fn parse_tag<S: AsRef<str>>(s: S) -> Result<Tag> {
    match _parse_tag(s.as_ref()) {
        IResult::Ok((i, o)) => {
            if i.len() > 0 {
                return Err(Error::ParseError(format!("Did not parse all of tag; leftover was \
                                                      {}",
                                                     i)));
            }
            Ok(o)
        }
        IResult::Err(NErr::Incomplete(e)) => {
            Err(Error::ParseError(format!("Tag {} was incomplete; error was {:?}", s.as_ref(), e)))
        }
        IResult::Err(NErr::Error((s, _))) => Err(Error::ParseError(format!("Parse error: {}", s))),
        IResult::Err(NErr::Failure((s, _))) => Err(Error::ParseError(format!("Parse failure: {}", s))),
    }
}

pub fn parse_str<S: AsRef<str>>(s: S) -> Result<Rule> {
    println!("parse_str: {}", s.as_ref());
    match _parse_str(s.as_ref()) {
        IResult::Ok((i, o)) => {
            if i.len() > 0 {
                return Err(Error::ParseError(format!("Did not parse all of rule; leftover was \
                                                      {}",
                                                     i)));
            }
            Ok(Rule::new(o))
        }
        IResult::Err(NErr::Incomplete(e)) => {
            Err(Error::ParseError(format!("Rule {} was incomplete; error was {:?}", s.as_ref(), e)))
        }
        IResult::Err(NErr::Error((s, _))) => Err(Error::ParseError(format!("Parse error: {}", s))),
        IResult::Err(NErr::Failure((s, _))) => Err(Error::ParseError(format!("Parse failure: {}", s))),
    }
}


#[cfg(test)]
mod tests {
    use nom::IResult;
    use std::collections::BTreeMap;
    use grammar::{Node, Rule};
    use tag::Tag;
    use super::{tag_as_node, text_as_node, tag_or_text, parse_str, action, modifier, parse_tag};
    #[test]
    fn test_tag_as_node() {
        let src = "#one#";
        match tag_as_node(src) {
            IResult::Ok((_, output)) => {
                match output {
                    Node::Tag(r) => assert_eq!(Tag::new("one"), r),
                    _ => panic!("somehow parsed {} as text??", src),
                }
            }
            _ => panic!("could not parse #one#"),
        }
    }

    #[test]
    fn test_text_as_node() {
        let src = "this is some text#";
        match text_as_node(src) {
            IResult::Ok((_, output)) => {
                match output {
                    Node::Text(o) => assert_eq!("this is some text".to_string(), o),
                    _ => panic!("somehow parsed {} as a Tag???!!", src),
                }
            }
            _ => panic!("could not parse {}", src),
        }
    }

    #[test]
    fn test_tag_or_text() {
        let src = "this is some #text#";
        match tag_or_text(src) {
            IResult::Ok((i, o)) => {
                match o {
                    Node::Text(t) => assert_eq!(t, "this is some ".to_string()),
                    _ => panic!("parsed {} as a tag?", src),
                }
                assert_eq!(i, "#text#".to_string());
            }
            _ => panic!("Could not parse {}", src),
        }
    }

    #[test]
    fn test_parse_str() {
        let src = "this is some #text# with normal stuff at the end";
        match parse_str(src) {
            Ok(o) => {
                assert_eq!(o,
                           Rule::new(vec![
                        Node::Text("this is some ".into()),
                        Node::Tag(Tag::new("text")),
                        Node::Text(" with normal stuff at the end".into()),
                        ]));
            }
            _ => panic!("colud not parse {}", src),
        }

        let src = "this #text# has multiple #tags# in it, and also ends with a #tag#";

        match parse_str(src) {
            Ok(o) => {
                assert_eq!(o,
                           Rule::new(vec![
                        Node::Text("this ".into()),
                        Node::Tag(Tag::new("text")),
                        Node::Text(" has multiple ".into()),
                        Node::Tag(Tag::new("tags")),
                        Node::Text(" in it, and also ends with a ".into()),
                        Node::Tag(Tag::new("tag")),
                        ]));
            }
            _ => panic!("could not parse {}", src),
        }
    }
    //
    #[test]
    fn test_parse_str_with_hash_dot() {
        let src = "#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the \
                   #heroPet# was always too #mood#.";
        match parse_str(src) {
            Ok(o) => {
                assert_eq!(o,
                           Rule::new(vec![
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
                            ]));
            }
            Err(e) => panic!("Could not parse '{}', error was '{}'", src, e),
        }
    }

    #[test]
    fn test_parse_action() {
        let src = "[one:#two#]";
        match action(src) {
            IResult::Ok((i, o)) => {
                assert_eq!(i, "");
                assert_eq!(o, ("one".to_string(), Rule::parse("#two#").unwrap()));
            }
            x => {
                println!("problem was: {:?}", x);
                panic!("Could not parse {} as an action", src);
            }
        }
    }

    #[test]
    fn test_parse_modifier() {
        let src = ".inQuotes#";
        match modifier(src) {
            IResult::Ok((i, o)) => {
                assert_eq!(i, "#");
                assert_eq!(o, "inQuotes");
            }
            _ => panic!("Could not parse {} as a modifier", src),
        }

        let src = ".s.inQuotes#";
        match modifier(src) {
            IResult::Ok((i, o)) => {
                assert_eq!(i, ".inQuotes#");
                assert_eq!(o, "s");
            }
            _ => panic!("Could not parse {} as a modifier", src),
        }

        named!(multiple_modifiers<&str, Vec<&str> >, many0!(modifier));

        let src = ".s.inQuotes#";
        match multiple_modifiers(src) {
            IResult::Ok((i, o)) => {
                assert_eq!(i, "#");
                assert_eq!(o[0], "s");
                assert_eq!(o[1], "inQuotes");
            }
            _ => panic!("Could not parse {} as multiple modifiers", src),
        }
    }

    #[test]
    fn test_parse_tag() {
        let src = "#tagname#";
        match parse_tag(src) {
            Ok(o) => {
                assert_eq!(o, Tag::new("tagname"));
            }
            _ => panic!("Could not parse {} as a tag", src),
        }

        let src = "#[one:#two#]tagname#";
        let mut actions = BTreeMap::new();
        actions.insert("one".to_string(), Rule::parse("#two#").unwrap());
        match parse_tag(src) {
            Ok(o) => {
                assert_eq!(o, Tag::new("tagname").with_actions(actions));
            }
            _ => panic!("Could not parse {} as a tag", src),
        }

        let src = "#[one:#two#][three:#four#]tagname#";
        let mut actions = BTreeMap::new();
        actions.insert("one".to_string(), Rule::parse("#two#").unwrap());
        actions.insert("three".to_string(), Rule::parse("#four#").unwrap());
        match parse_tag(src) {
            Ok(o) => {
                assert_eq!(o, Tag::new("tagname").with_actions(actions));
            }
            _ => panic!("Could not parse {} as a tag", src),
        }

        let src = "#tagname.s#";
        match parse_tag(src) {
            Ok(o) => {
                assert_eq!(o, Tag::new("tagname").with_modifiers(vec!["s"]));
            }
            Err(e) => panic!("Could not parse {} as a tag: {:?}", src, e),
        }

        let src = "#tagname.s.capitalize#";
        match parse_tag(src) {
            Ok(o) => {
                assert_eq!(o,
                           Tag::new("tagname").with_modifiers(vec!["s", "capitalize"]));
            }
            _ => panic!("Could not parse {} as a tag", src),
        }

        let src = "#[one:#two#][three:#four#]tagname.s.capitalize#";
        let mut actions = BTreeMap::new();
        actions.insert("one".to_string(), Rule::parse("#two#").unwrap());
        actions.insert("three".to_string(), Rule::parse("#four#").unwrap());
        match parse_tag(src) {
            Ok(o) => {
                assert_eq!(o,
                           Tag::new("tagname")
                               .with_actions(actions)
                               .with_modifiers(vec!["s", "capitalize"]));
            }
            _ => panic!("Could not parse {} as a tag", src),
        }
    }
}
