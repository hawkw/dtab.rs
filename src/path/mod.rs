use std::{convert, error, fmt, iter, ops, str, };
use std::ascii::AsciiExt;
use regex::Regex;

pub mod prefix;
pub use self::prefix::Prefix;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Label<'t>(pub &'t str);

#[derive(Copy, Clone, Debug)]
pub enum LabelError<'a> {
    InvalidCharacter { ch: &'a str, at: usize, elem: &'a str },
    NonAscii { ch: &'a str, at: usize },
}

impl<'a> LabelError<'a> {
    #[inline] fn non_ascii(s: &'a str) -> Result<&'a str, Self> {
        s.chars().position(|c| !c.is_ascii())
         .map(|i| Err(LabelError::NonAscii { ch: &s[i .. i + 1], at: i }) )
         .unwrap_or_else(|| Ok(s))

    }

    #[inline] fn invalid_char(s: &'a str) -> Result<&'a str, Self> {
        lazy_static! {
            static ref INVALID_REGEX: Regex =
                Regex::new("([^0-9A-Za-z:.#$%-_])")
                    .expect("Failed to compile invalid char regex!");
        }
        INVALID_REGEX.find(s)
            .map(|m| Err(LabelError::InvalidCharacter {
                ch: &s[m.start() .. m.end()],
                at: m.start(), elem: s })
            )
            .unwrap_or_else(|| Ok(s))
    }
}

impl<'t> convert::TryFrom<&'t str> for Label<'t> {
    type Error = LabelError<'t>;
    fn try_from(s: &'t str) -> Result<Self, Self::Error> {
        use regex::Regex;
        lazy_static! {
            static ref LABEL_REGEX: Regex =
                Regex::new(r"(\\x[a-f0-9][a-f0-9]|[0-9A-Za-z:.#$%-_])+")
                    .expect("Failed to compile label-parsing regex!");
        }
        if LABEL_REGEX.is_match(s) {
            Ok(Label(s))
        } else {
            Err(LabelError::non_ascii(s)
                .and_then(LabelError::invalid_char)
                .expect_err("Unknown parsing error"))
        }

    }
}

pub struct Path<'t>(pub Vec<&'t [u8]>);

impl<'t> Path<'t> {
    pub fn append<'b, T>(&mut self, path: T) -> Result<&mut Self, LabelError>
    where T: convert::Into<&'b [u8]>
        , 'b: 't {
        self.0.push(path.into());
        Ok(self)
    }
}

impl<'a, 'b, R> ops::Div<&'b R> for Path<'a>
where R: convert::AsRef<[u8]>
    , 'b: 'a
    {
    type Output = Self;
    fn div(mut self, rhs: &'b R) -> Self {
        self.0.push(rhs.as_ref());
        self
    }

}

impl<'a, 'b, R> ops::Div<R> for &'a mut Path<'a>
where R: convert::Into<&'b [u8]>
    , 'b: 'a
    {
    type Output = Self;
    fn div(self, rhs: R) -> Self {
        self.append(rhs)
            .expect("Error appending to path from iterator")
    }

}

impl <'a, 'b, T> iter::Extend<T> for Path<'a>
where T: convert::Into<&'b [u8]>
    , 'b: 'a
    {
    fn extend<I>(&mut self, iter: I)
    where I: iter::IntoIterator<Item=T> {
        for elem in iter {
            self.append(elem)
                .expect("Error extending path from iterator");
        }
    }
}

//impl<'a, 'b, I> convert::From<I> for Path<'a>
//where I: convert::Into<&'b [u8]>
//    , 'b: 'a
//    {
//    #[inline] fn from(i: I) -> Path<'a> {
//        Path(vec![i.into()])
//    }
//}

impl<'a, A> convert::From<&'a A> for Path<'a>
where A: convert::AsRef<[u8]>
    , A: 'a {
    #[inline] fn from(bytes: &'a A) -> Path<'a> {
        Path(vec![bytes.as_ref()])
    }
}
//
//impl<'a, A> convert::From<&'a A> for Path<'a>
//where A: convert::AsRef<[u8]>
//    , A: 'a {
//    #[inline] fn from(bytes: &'a A) -> Path<'a> {
//        Path(vec![bytes.as_ref()])
//    }
//}


impl <'a> fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ref elem in &self.0 {
            write!(f, "/{}", unsafe {
                // validity of ascii chars should already be checked
                // when elements are added to a path...
                str::from_utf8_unchecked(elem)
            })?;
        }
        Ok(())
    }
}



impl<'t> fmt::Display for LabelError<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LabelError::InvalidCharacter { ch, at, elem } =>
              write!( f
                    , "Invalid character {ch:?} at position {at} in {elem:?}."
                    , ch = ch
                    , at = at
                    , elem = elem ),
            LabelError::NonAscii { ch, at } =>
              write!( f
                    , "Non-ASCII character {ch:?} at position {at}."
                    , ch = ch
                    , at = at)
        }
    }
}

impl<'a> error::Error for LabelError<'a> {
    fn description(&self) -> &str {
        match *self {
            LabelError::InvalidCharacter { .. } =>  "invalid character",
            LabelError::NonAscii { .. } => "non-ASCII character"
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn path_slash_operator() {
        let path = Path::from(b"aaaa") / b"bbbb";
        assert_eq!(format!("{}", path), "/aaaa/bbbb")

    }
}
