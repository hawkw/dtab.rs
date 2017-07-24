//! Finagle/linkerd Name Trees.
//!
//! This implementation is based loosely on Finagle's [Scala implementation].
//!
//! [`NameTree`]s can be parsed from strings; or they may be constructed
//! programmatically using a set of operators. These operators provide a
//! type-safe DSL for constructing correct `NameTree`s.
//!
//! # Examples
//!
//! These examples are taken from linkerd's [documentation] on dtabs.
//!
//! Suppose we had the simple dtab
//!
//! ```notrust
//! /iceCreamStore => /smitten;
//! ```
//!
//! We could construct the corresponding [`Dentry`] using the following Rust
//! expression:
//!
//! ```
//! #![feature(try_from)]
//! # fn main() {
//! use dtab::{NameTree, Prefix};
//! use std::convert::TryFrom;
//!
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >> "/smitten";
//! assert_eq!("/iceCreamStore => /smitten;", &dentry.to_string());
//! # }
//! ```
//!
//! Take note of the following:
//!
//! + The right hand side of a `NameTree` operator must be of the type
//!   `NameTree<T>`, but the left hand side may be of any type
//!   `R: convert::Into<NameTree<T>>`, due to Rust's trait implementation
//!   rules. This means that we must explicitly call `NameTree::from` for the
//!   first path in the tree, but we can then use string literals for every
//!   other element, as `NameTree<String>` implements `convert::From<&str>`.
//! + The `>>` operator is used in place of `=>` to construct a [`Dentry`].
//!   `=>` is a reserved word in Rust, but `>>` is [an overridable operator].
//!
//! The `|` operator can be used to programmatically construct alternation
//! expressions. For example:
//!
//! ```notrust
//! /iceCreamStore => /humphrys | /smitten;
//! ```
//!
//! becomes
//!
//! ```
//! #![feature(try_from)]
//! # fn main() {
//! use dtab::{NameTree, Prefix};
//! use std::convert::TryFrom;
//!
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >>
//!              (NameTree::from("/humphrys") | "/smitten");
//! assert_eq!("/iceCreamStore => /humphrys | /smitten;", &dentry.to_string());
//! # }
//! ```
//!
//! These alternation expressions can have any number of alternates, as in:
//!
//! ```
//! #![feature(try_from)]
//! # fn main() {
//! use dtab::{NameTree, Prefix};
//! use std::convert::TryFrom;
//!
//! let dest = NameTree::from("/humphrys") | "/smitten" | "/birite"
//!                   | "/three-twins";
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >> dest;
//! assert_eq!(
//!   "/iceCreamStore => /humphrys | /smitten | /birite | /three-twins;"
//! , &dentry.to_string()
//! );
//! # }
//! ```
//!
//! Union expressions can be constructed using the `&` operator:
//!
//! ```
//! #![feature(try_from)]
//! # fn main() {
//! use dtab::{NameTree, Prefix};
//! use std::convert::TryFrom;
//!
//! let dest = NameTree::from("/smitten") & "/humphrys";
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >> dest;
//!
//! assert_eq!( "/iceCreamStore => 0.5 * /smitten & 0.5 * /humphrys;"
//!            , &dentry.to_string());
//! # }
//! ```
//!
//! Note that if no weight is supplied, the value of [`DEFAULT_WEIGHT`], 0.5,
//! will be used.
//!
//! Weighted unions can be constructed using the `*` operator:
//!
//! ```
//! #![feature(try_from)]
//! # fn main() {
//! use dtab::{NameTree, Prefix};
//! use std::convert::TryFrom;
//! use dtab::nametree::W;
//!
//! let dest = W(0.7) * "/smitten" & W(0.3) * "/humphrys";
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >> dest;
//! assert_eq!( "/iceCreamStore => 0.7 * /smitten & 0.3 * /humphrys;"
//!            , &dentry.to_string());
//! # }
//! ```
//!
//! [`W()`] is a [newtype] used to allow the implementation of custom operators
//! on `f64`. It's only used for constructing weighted `NameTree` expressions.
//! The name `W` was chosen to keep the `NameTree` DSL from becoming too wordy.
//!
//!
//! Finally, the strings `"~"`, `"!"`, and `"$"` will convert into the negation,
//! failure, and empty `NameTree` nodes, rather than leaf nodes:
//!
//! ```
//! #![feature(try_from)]
//! # fn main() {
//! use dtab::{NameTree, Prefix};
//! use std::convert::TryFrom;
//!
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >>
//!              (NameTree::from("~") | "/smitten");
//! assert_eq!( "/iceCreamStore => ~ | /smitten;"
//!            , &dentry.to_string());
//!
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >>
//!              (NameTree::from("/smitten") | "!");
//! assert_eq!( "/iceCreamStore => /smitten | !;"
//!            , &dentry.to_string());
//! # }
//! ```
//!
//! Note that this only works when the leaf type of the `NameTree` is `String`.
//!
//! The `NameTree` variants `Neg`, `Fail`, and `Empty` can also be used
//! explictly:
//!
//! ```
//! #![feature(try_from)]
//! # fn main() {
//! use dtab::{NameTree, Prefix};
//! use std::convert::TryFrom;
//!
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >>
//!             (NameTree::Neg | "/smitten");
//! assert_eq!( "/iceCreamStore => ~ | /smitten;"
//!            , &dentry.to_string());
//!
//! let dentry = Prefix::try_from("/iceCreamStore").unwrap() >>
//!              (NameTree::from("/smitten") | NameTree::Fail);
//! assert_eq!( "/iceCreamStore => /smitten | !;"
//!            , &dentry.to_string());
//! # }
//! ```
//!
//! [`Dentry`]: ../struct.Dentry.html
//! [`NameTree`]: enum.NameTree.html
//! [`W()`]: struct.W.html
//! [Scala implementation]: https://github.com/twitter/finagle/blob/master/finagle-core/src/main/scala/com/twitter/finagle/NameTree.scala
//! [documentation]: https://linkerd.io/in-depth/dtabs/
//! [an overridable operator]: https://doc.rust-lang.org/std/ops/trait.Shr.html
//! [newtype]: https://aturon.github.io/features/types/newtype.html

use std::{ops, convert, fmt};
use super::Dentry;
use self::NameTree::*;
pub const DEFAULT_WEIGHT: f64 = 0.5;

/// Name trees represent a composite name whose interpretation is subject to
/// Finagle's interpretation rules
#[derive(Clone, PartialEq, Debug)]
pub enum NameTree<T> { Leaf(T)
                     , Union(Weighted<T>, Weighted<T>)
                     , Alt(Box<NameTree<T>>, Box<NameTree<T>>)
                     , Neg
                     , Empty
                     , Fail
                     }

impl<T> NameTree<T> {
    #[inline] pub fn weighted(self, weight: f64) -> Weighted<T> {
        Weighted { weight: weight, tree: Box::new(self)}
    }
}

impl<'a> convert::From<&'a str> for NameTree<String> {
    #[inline] fn from(s: &'a str) -> Self {
      match s { "~" => Neg
              , "!" => Fail
              , "$" => Empty
                // TODO: validate paths?
              , path => Leaf(path.to_string())
              }

    }
}

impl<T> fmt::Display for NameTree<T>
where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Leaf(ref value) => write!(f, "{}", value)
          , Union(ref left, ref right) => write!(f, "{} & {}", left, right)
          , Alt(ref left, ref right) => write!(f, "{} | {}", left, right)
          , Fail => write!(f, "!")
          , Neg => write!(f, "~")
          , Empty => write!(f, "$")
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Weighted<T> { weight: f64, tree: Box<NameTree<T>> }

impl<T> fmt::Display for Weighted<T>
where T: fmt::Display {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} * {}", self.weight, self.tree)
    }

}
//
// pub trait NameTree {
// }
// pub struct Alt(Box<NameTree>, Box<NameTree>);
// impl NameTree for Alt {}

pub struct W(pub f64);


impl<T, R> ops::BitAnd<R> for NameTree<T>
where R: convert::Into<NameTree<T>> {
    type Output = Self;
    #[inline] fn bitand(self, rhs: R) -> Self {
        Union( self.weighted(DEFAULT_WEIGHT)
             , rhs.into().weighted(DEFAULT_WEIGHT))
    }
}

impl<T> ops::BitAnd for Weighted<T> {
    type Output = NameTree<T>;
    #[inline] fn bitand(self, rhs: Self) -> NameTree<T> {
        Union(self, rhs)
    }
}

impl<T, R> ops::BitOr<R> for NameTree<T>
where R: convert::Into<NameTree<T>> {
    type Output = Self;
    #[inline] fn bitor(self, rhs: R) -> Self {
        Alt(Box::new(self), Box::new(rhs.into()))
    }
}

// impl<T> ops::Mul<NameTree<T>> for W {
//
//     type Output = Weighted<T>;
//     #[inline] fn mul(self, rhs: NameTree<T>) -> Self::Output {
//         let W(w) = self;
//         Weighted { weight: w, tree: Box::new(rhs) }
//     }
// }

impl<R> ops::Mul<R> for W
where R: convert::Into<NameTree<String>> {

    type Output = Weighted<String>;
    #[inline] fn mul(self, rhs: R) -> Self::Output {
        let W(w) = self;
        Weighted { weight: w, tree: Box::new(rhs.into()) }
    }
}

#[cfg(feature = "serialize")]
use serde::ser::{Serializer};
#[cfg(feature = "serialize")]
pub fn serialize<S>(name_tree: &NameTree<String>, serializer: S)
                    -> Result<S::Ok, S::Error>
where S: Serializer {
    serializer.serialize_str(&format!("{}", name_tree))
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::NameTree::*;
    use std::convert::From;



    #[test]
    fn simple_alt() {
        let t = NameTree::from("/humphrys") | "/smitten";
        assert_eq!(t, Alt( Box::new(NameTree::from("/humphrys"))
                         , Box::new(NameTree::from("/smitten"))
                         )
                  );
    }

    #[test]
    fn multiple_alt() {
        let t = NameTree::from("/humphrys") | "/smitten"
                                            | "/birite"
                                            | "/three-twins";
        assert_eq!(t,
            Alt(
                Box::new( Alt(
                    Box::new( Alt(
                        Box::new(NameTree::from("/humphrys"))
                      , Box::new(NameTree::from("/smitten"))
                    ))
                  , Box::new(NameTree::from("/birite"))
                ))
              , Box::new(NameTree::from("/three-twins"))
            )
        );
    }


    #[test]
    fn neg_alt() {
        let t = NameTree::from("~") | "/smitten";
        assert_eq!( t
                  , Alt( Box::new(Neg)
                       , Box::new(Leaf("/smitten".to_string()))
                       )
                  );
    }

    #[test]
    fn fail_alt() {
        let t = NameTree::from("/smitten") | "!";
        assert_eq!( t
                  , Alt( Box::new(Leaf("/smitten".to_string()))
                       , Box::new(Fail)
                       )
                  );
    }

    #[test]
    fn simple_union() {
        let t = NameTree::from("/humphrys") & "/smitten";
        assert_eq!( t
                  , Union( W(0.5) * Leaf("/humphrys".to_string())
                         , W(0.5) * Leaf("/smitten".to_string())
                         )
        );
    }

    #[test]
    fn simple_weighted_union() {
        let t = W(0.7) * "/humphrys" & W(0.3) * "/smitten";
        assert_eq!( t
                  , Union( W(0.7) * Leaf("/humphrys".to_string())
                         , W(0.3) * Leaf("/smitten".to_string())
                         )
        );
    }

}

// impl ops::BitOr for NameTree {
//
// }

// mod parser {
//   use super::NameTree;
//   use std::convert::TryFrom;
//
//   named!{ weighted<&str, (f64, NameTree)>,
//     do_parse!(w: map_res!(re_match!(r"[0-9]*\.?[0-9]+"), f64::from_str) >>
//               tag!("*") >>
//               t: tree   >>
//               (w, t)
//     )
//   }
//   named!{union<&str, NameTree>,
//     do_parse!(w: map_res!(re_match!(r"[0-9]*\.?[0-9]+"), f64::from_str) >>
//               tag!("*") >>
//               t: tree   >>
//               (w, t)
//     )
//   }
//   named!{pub tree<&str, NameTree>,
//     alt!(name |
//     )
//   }
//   named!(name<&str, NameTree>,
//     alt!( map!(tag!("!"), |_| { NameTree::Fail })  |
//           map!(tag!("~"), |_| { NameTree::Neg })   |
//           map!(tag!("$"), |_| { NameTree::Empty }) )
//   );
//
// }
