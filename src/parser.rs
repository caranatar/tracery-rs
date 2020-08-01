use crate::Error;
use crate::grammar::Rule;
use crate::tag::Tag;

pub fn parse_str<S: AsRef<str>>(s: S) -> Result<Rule, Error> {
    unimplemented!()
}

pub fn parse_tag<S: AsRef<str>>(s: S) -> Result<Tag, Error> {
    unimplemented!()
}
