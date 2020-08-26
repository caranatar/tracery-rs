use crate::tag::Tag;
use crate::Flatten;
use crate::Grammar;
use crate::Result;

use std::collections::BTreeMap;

/// Represents a part of a single expandable string
///
/// This is used to represent both the plain text, and the expandable text sections of a string.
///
/// # Example
///
/// ```
/// # use tracery::{Node, Result, Rule, Tag};
/// # fn main() -> Result<()> {
/// let nodes = vec![
///     Node::Tag(Tag::new("one")),
///     Node::Text(" is the loneliest number".to_string()),
/// ];
/// let rule = Rule::new(nodes);
///
/// assert_eq!(Rule::parse("#one# is the loneliest number")?, rule);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Node {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn conversion() -> Result<()> {
        let tag = Tag::parse("#a#")?;
        assert_eq!(Node::Tag(tag.clone()), Node::from(tag));
        
        let text = "abc".to_string();
        assert_eq!(Node::Text(text.clone()), Node::from(text));
        
        Ok(())
    }
}
