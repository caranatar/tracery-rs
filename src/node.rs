use crate::tag::Tag;
use crate::Flatten;
use crate::Grammar;
use crate::Result;

use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Node {
    /// A tag (a key surrounded by '#'s)
    Tag(Tag),
    /// Plain text
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

impl Flatten for Node {
    fn flatten<R: ?Sized + rand::Rng>(
        &self,
        grammar: &Grammar,
        overrides: &mut BTreeMap<String, String>,
        rng: &mut R,
    ) -> Result<String> {
        match self {
            Node::Tag(ref tag) => tag.flatten(grammar, overrides, rng),
            Node::Text(ref s) => s.flatten(grammar, overrides, rng),
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
