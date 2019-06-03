//! # Serde Regex
//!
//! [Documentation](https://docs.rs/serde_regex) |
//! [Github](https://github.com/tailhook/serde-regex) |
//! [Crate](https://crates.io/crates/serde_regex)
//!
//! A (de)serializer for `regex::Regex`
//!
//! # Example
//!
//! ```rust
//!
//! use regex::Regex;
//! use serde::{Deserialize, Serialize};
//! use serde_derive::{Serialize, Deserialize};
//!
//!
//! #[derive(Serialize, Deserialize)]
//! struct Timestamps {
//!     #[serde(with = "serde_regex")]
//!     pattern: Regex,
//! }
//!
//! #
//! # fn main() {}
//! ```
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use regex::Regex;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::de::{Error, Visitor};
use serde::{ser::SerializeSeq, Serialize, Serializer};
use serde::{Deserialize, Deserializer, de::SeqAccess};

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `Regex`
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Serde<T>(pub T);

struct RegexVisitor;
struct RegexVecVisitor;

impl<'a> Visitor<'a> for RegexVisitor {
    type Value = Serde<Regex>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid regular expression")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Regex::new(value).map_err(E::custom).map(Serde)
    }
}

impl<'a> Visitor<'a> for RegexVecVisitor {
    type Value = Serde<Vec<Regex>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid sequence")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'a>,
    {
        let mut vec = match seq.size_hint() {
            Some(size) => Vec::with_capacity(size),
            None => Vec::new(),
        };
        while let Some(Serde(el)) = seq.next_element()? {
            vec.push(el);
        }
        return Ok(Serde(vec));
    }
}

impl<'de> Deserialize<'de> for Serde<Option<Regex>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<Regex>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<Regex>>::deserialize(d)? {
            Some(Serde(regex)) => Ok(Serde(Some(regex))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Regex> {
    fn deserialize<D>(d: D) -> Result<Serde<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(RegexVisitor)
    }
}

impl<'de> Deserialize<'de> for Serde<Vec<Regex>> {
    fn deserialize<D>(d: D) -> Result<Serde<Vec<Regex>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_seq(RegexVecVisitor)
    }
}

/// Deserialize function, see crate docs to see how to use it
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    Serde<T>: Deserialize<'de>,
{
    Serde::deserialize(deserializer).map(|x| x.0)
}

/// Deserialize function, see crate docs to see how to use it
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    for<'a> Serde<&'a T>: Serialize,
{
    Serde(value).serialize(serializer)
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Serde<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Serde<T> {
    /// Consumes the `Serde`, returning the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Serde<T> {
    fn from(val: T) -> Serde<T> {
        Serde(val)
    }
}

impl<'a> Serialize for Serde<&'a Regex> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl Serialize for Serde<Regex> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl<'a> Serialize for Serde<&'a Option<Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            &Some(ref value) => serializer.serialize_some(&Serde(value)),
            &None => serializer.serialize_none(),
        }
    }
}

impl Serialize for Serde<Option<Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl Serialize for Serde<Vec<Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl<'a> Serialize for Serde<&'a Vec<Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for element in self.0 {
            seq.serialize_element(&Serde(element))?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod test {
    use serde_json::{json, from_value};
    use regex::Regex;
    use crate::Serde;

    #[test]
    fn test_vec() -> Result<(), Box<std::error::Error>> {
        let json = json!(["a.*b", "c?d"]);
        let vec: Serde<Vec<Regex>> = from_value(json)?;
        assert!(vec.0[0].as_str() == "a.*b");
        assert!(vec.0[1].as_str() == "c?d");
        assert!(vec.len() == 2);
        Ok(())
    }
}
