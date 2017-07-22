//! A library for interpreting & manipulating Finagle/linkerd [dtab]s.
//!
//! The goal of this library is to provide a DSL for constructing and
//! manipulating dtabs that uses Rust's type system to ensure that invalid
//! dtabs cannot be represented, rather than just representing them as strings.
//!
//! [dtab]: https://linkerd.io/in-depth/dtabs/
#![feature(try_from)]
#![feature(ascii_ext)]
#[cfg(test)]
#[macro_use] extern crate pretty_assertions;

extern crate regex;
#[macro_use] extern crate lazy_static;

extern crate serde;
#[macro_use] extern crate serde_derive;
//
// #[macro_use] extern crate nom;

// extern crate regex;

use std::fmt;

pub mod nametree;
pub mod path;

pub use self::nametree::*;
pub use self::path::{Prefix, Path};


/// Macro for constructing a [`Dentry`].
///
/// This allows us to make [`Dentry`]s somewhat more ergonomically
/// than using the `>>` operator (since it doesn't require
/// the use of grouping symbols for the destination side of the
/// Dentry when using the [`NameTree`] DSL).
///
/// The macro also allows the use of `=>` rather than `>>`, as to
/// more closely match the dtab synbtax.
///
/// # Examples
///
/// ```
/// #![feature(try_from)]
/// #[macro_use] extern crate dtab;
/// # fn main() {
/// use dtab::NameTree;
///
/// let dentry = dentry!( "/iceCreamStore" =>
///     NameTree::from("/smitten") | "/humphrys" | "/birite" | "/three-twins"
///  ).unwrap();
///
/// assert_eq!(
///   "/iceCreamStore => /smitten | /humphrys | /birite | /three-twins;"
/// , &dentry.to_string()
/// );
/// # }
/// ```
///
#[macro_export]
macro_rules! dentry {
  ($src: expr => $dst: expr ) => ({
    use std::convert::TryFrom;
    $crate::path::Prefix::try_from($src)
        .map(|src| $crate::Dentry {
                prefix: src, dst: $dst
            })

    })
}

/// Convenience macro for making [`Dtab`]s.
///
/// # Examples
///
/// ```
/// #![feature(try_from)]
/// #[macro_use] extern crate dtab;
/// # fn main() {
/// use dtab::NameTree;
///
/// let dtab = dtab![
///   "/smitten"       => NameTree::from("/USA/CA/SF/Harrison/2790");
///   "/iceCreamStore" => NameTree::from("/humphrys") | "/smitten";
/// ].unwrap();
///
///
/// assert_eq!( &format!("{}", dtab)
///           , "/smitten => /USA/CA/SF/Harrison/2790;\n\
///              /iceCreamStore => /humphrys | /smitten;\n"
///           );
/// # }
/// ```
///
/// [`Dtab`]: type.Dtab.html
#[macro_export]
macro_rules! dtab {
  ($($src: expr => $dst: expr ;)+) => (
    vec![ $(dentry!($src => $dst)),+ ].into_iter()
        .collect::<Result<Vec<$crate::Dentry>,$crate::path::LabelError>>()
        .map($crate::Dtab)
  )
}
/// A `dtab` (delegation table) comprises a sequence of delegation rules.
#[derive(Debug, Clone, Serialize)]
pub struct Dtab<'a>(pub Vec<Dentry<'a>>);

impl<'a> fmt::Display for Dtab<'a> {
    #[inline] fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter()
            .fold( Ok(())
                 , |r, ref entry| r.and_then(|_| {
                        write!(f, "{}\n", entry)
                    }))

    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Dentry<'prefix> {
    #[serde(serialize_with ="path::prefix::serialize")]
    pub prefix: Prefix<'prefix>
  , #[serde(serialize_with ="nametree::serialize")]
    pub dst: NameTree<String>
}

impl<'a> fmt::Display for Dentry<'a> {
    #[inline] fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} => {};", self.prefix, self.dst)
    }
}

use std::{convert, ops};

impl<'a, R> ops::Shr<R> for Prefix<'a>
where R: convert::Into<NameTree<String>> {
    type Output = Dentry<'a>;
    #[inline] fn shr(self, rhs: R) -> Self::Output {
        Dentry { prefix: self, dst: rhs.into() }
    }
}
