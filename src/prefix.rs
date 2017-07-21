use std::str::FromStr;
use std::fmt;

use regex::Regex;

#[derive(Debug, Eq, PartialEq)]
pub struct Prefix(Vec<Elem>);

impl FromStr for Prefix {
    type Err = ParseElemErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('/').filter(|s| s != &"")
         .map(Elem::from_str)
         .collect::<Result<Vec<Elem>, ParseElemErr>>()
         .map(Prefix)
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ref elem in &self.0 {
            write!(f, "/{}", elem)?;
        }
        Ok(())
    }
}


// TODO: record invalid char position
#[derive(Clone, Debug)]
pub struct ParseElemErr { failed: String }

#[derive(Debug, Eq, PartialEq)]
pub enum Elem {
    /// a label
    Label(String)
  , /// the `*` character
    AnyElem
}

impl fmt::Display for Elem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                Elem::Label(ref s) => s
              , Elem::AnyElem => "*"
            })
    }
}

/// TODO: consider using `TryFrom` instead, so that `Label` can be borrowed
impl FromStr for Elem {
    type Err = ParseElemErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref LABEL_REGEX: Regex =
                Regex::new(r"(\\x[a-f0-9][a-f0-9]|[0-9A-Za-z:.#$%-_])+")
                    .expect("Failed to compile label-parsing regex!");
        }
        if s == "*" { Ok(Elem::AnyElem) }
        else if LABEL_REGEX.is_match(s) {
            Ok(Elem::Label(s.to_owned()))
        } else {
            Err(ParseElemErr{ failed: s.to_owned() })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Elem::*;

    #[test]
    fn test_parse_1() {
        let prefix: Prefix = "/foo/bar/baz".parse().unwrap();
        assert_eq!( prefix
                  , Prefix(vec![ Label("foo".to_string())
                               , Label("bar".to_string())
                               , Label("baz".to_string())
                               ])
                  )
    }
    #[test]
    fn test_parse_2() {
        let prefix: Prefix = "/foo/*/bar/baz".parse().unwrap();
        assert_eq!( prefix
                  , Prefix(vec![ Label("foo".to_string())
                               , AnyElem
                               , Label("bar".to_string())
                               , Label("baz".to_string())
                               ])
                  )
    }
    #[test]
    fn test_parse_empty() {
        let prefix: Prefix = "/".parse().unwrap();
        assert_eq!( prefix
                  , Prefix(vec![]))
    }
}
