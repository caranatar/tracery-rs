use crate::tag::Tag;
use crate::Execute;
use crate::Grammar;
use crate::Result;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Node {
    /// A tag (a key surrounded by '#'s)
    Tag(Tag),
    /// Plain text
    Text(String),
}

impl Node {
    pub(crate) fn text(&self) -> Option<&String> {
        match self {
            Node::Tag(_) => None,
            Node::Text(s) => Some(s),
        }
    }
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

impl Execute for Node {
    fn execute<R: ?Sized + rand::Rng>(&self, grammar: &mut Grammar, rng: &mut R) -> Result<String> {
        match self {
            Node::Tag(ref tag) => tag.execute(grammar, rng),
            Node::Text(ref s) => Ok(s.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_tag;

    #[test]
    fn conversion() -> Result<()> {
        let tag = parse_tag("#a#")?;
        assert_eq!(Node::Tag(tag.clone()), Node::from(tag));

        let text = "abc".to_string();
        assert_eq!(Node::Text(text.clone()), Node::from(text));

        Ok(())
    }
}
