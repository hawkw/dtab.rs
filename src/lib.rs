//! A library for interpreting & manipulating Finagle/linkerd [dtab]s.
//!
//! The goal of this library is to provide a DSL for constructing and
//! manipulating dtabs that uses Rust's type system to ensure that invalid
//! dtabs cannot be represented, rather than just representing them as strings.
//!
//! [dtab]: https://linkerd.io/in-depth/dtabs/
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

extern crate serde;
#[macro_use] extern crate serde_derive;
//
// #[macro_use] extern crate nom;

// extern crate regex;

use std::fmt;

pub mod nametree;
pub mod path;

pub use self::nametree::*;


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
/// # #[macro_use] extern crate dtab;
/// # fn main() {
/// use dtab::NameTree;
///
/// let dentry = dentry!( "/iceCreamStore" =>
///     NameTree::from("/smitten") | "/humphrys" | "/birite" | "/three-twins" );
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
  ($src: expr => $dst: expr ) => ($crate::Dentry {
      prefix: $crate::NameTree::from($src), dst: $dst
  })
}

/// Convenience macro for making [`Dtab`]s.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate dtab;
/// # fn main() {
/// use dtab::NameTree;
///
/// let dtab = dtab![
///   "/smitten"       => NameTree::from("/USA/CA/SF/Harrison/2790");
///   "/iceCreamStore" => NameTree::from("/humphrys") | "/smitten";
/// ];
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
    $crate::Dtab(vec![ $(dentry!($src => $dst)),+ ])
  )
}
/// A `dtab` (delegation table) comprises a sequence of delegation rules.
#[derive(Debug, Clone, Serialize)]
pub struct Dtab(pub Vec<Dentry>);

impl fmt::Display for Dtab {
    #[inline] fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter()
            .fold( Ok(())
                 , |r, ref entry| r.and_then(|_| {
                        write!(f, "{}\n", entry)
                    }))

    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Dentry {
    #[serde(serialize_with ="nametree::serialize")]
    pub prefix: NameTree<String>
  , #[serde(serialize_with ="nametree::serialize")]
    pub dst: NameTree<String>
}

impl fmt::Display for Dentry {
    #[inline] fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} => {};", self.prefix, self.dst)
    }
}
