use std::{fmt, ops, convert};

use super::{Label, LabelError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prefix<'t>(Vec<Elem<'t>>);

impl<'t> fmt::Display for Prefix<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ref elem in &self.0 {
            write!(f, "/{}", elem)?;
        }
        Ok(())
    }
}

use serde::ser::{Serializer};
pub fn serialize<'t, S>(prefix: &Prefix<'t>, serializer: S)
                    -> Result<S::Ok, S::Error>
where S: Serializer {
    serializer.serialize_str(&format!("{}", prefix))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Elem<'t> {
    /// a label
    Label(Label<'t>)
  , /// the `*` character
    AnyElem
}

impl<'t> fmt::Display for Elem<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                Elem::Label(Label(ref s)) => s
              , Elem::AnyElem => "*"
            })
    }
}


#[cfg(feature = "parse")]
impl<'t> convert::TryFrom<&'t str> for Elem<'t> {
    type Error = LabelError<'t>;
    fn try_from(value: &'t str) -> Result<Self, Self::Error> {
        if value == "*" { Ok(Elem::AnyElem) }
        else {
            Label::try_from(value).map(Elem::Label)
        }
    }
}

#[cfg(feature = "parse")]
impl<'t> convert::TryFrom<&'t str> for Prefix<'t>
//where A: convert::AsRef<str>
//    , A: 't
    {
    type Error = LabelError<'t>;
    fn try_from(value: &'t str) -> Result<Self, Self::Error> {
        value.split('/')
            .filter(|s| !s.is_empty())
            .map(|ref s| Elem::try_from(s))
            .collect::<Result<Vec<Elem<'t>>, Self::Error>>()
            .map(Prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::path::Label;

    #[cfg(feature = "parse")]
    use std::convert::TryFrom;

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_1() {
        let prefix = Prefix::try_from("/foo/bar/baz").unwrap();
        assert_eq!( prefix
                  , Prefix(vec![ Elem::Label(Label("foo"))
                               , Elem::Label(Label("bar"))
                               , Elem::Label(Label("baz"))
                               ])
                  )
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_2() {
        let prefix = Prefix::try_from("/foo/*/bar/baz").unwrap();
        assert_eq!( prefix
                  , Prefix(vec![ Elem::Label(Label("foo"))
                               , Elem::AnyElem
                               , Elem::Label(Label("bar"))
                               , Elem::Label(Label("baz"))
                               ])
                  )
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_empty() {
        let prefix = Prefix::try_from("/").unwrap();
        assert_eq!( prefix
                  , Prefix(vec![]))
    }
}
