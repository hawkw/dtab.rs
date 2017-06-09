use std::{convert, fmt, iter, ops};

pub struct Path<'bytes>(pub Vec<&'bytes [u8]>);

impl<'bytes> Path<'bytes> {
    pub fn append<'b, T>(&mut self, path: T) -> Result<&mut Self, PathError>
    where T: convert::Into<&'b [u8]>
        , 'b: 'bytes {
        self.0.push(path.into());
        Ok(self)
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
