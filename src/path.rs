use std::{convert, fmt, iter, ops, str};

pub struct Path<'bytes>(pub Vec<&'bytes [u8]>);

impl<'bytes> Path<'bytes> {
    pub fn append<'b, T>(&mut self, path: T) -> Result<&mut Self, PathError>
    where T: convert::Into<&'b [u8]>
        , 'b: 'bytes {
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

pub enum PathError<'bytes> {
    InvalidCharacter { ch: char, at: usize, elem: &'bytes str }
}

impl<'bytes> fmt::Debug for PathError<'bytes> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PathError::InvalidCharacter { ch, at, elem } =>
              write!( f
                    , "Invalid character {ch:?} at position {at} in {elem:?}.`"
                    , ch = ch
                    , at = at
                    , elem = elem )
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
