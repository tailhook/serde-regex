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
//! #[macro_use]
//! extern crate serde_derive;
//!
//! extern crate serde;
//! extern crate regex;
//! extern crate serde_regex;
//!
//! use regex::Regex;
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
extern crate regex;
extern crate serde;

#[cfg(test)]
extern crate serde_json;

use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `Regex`
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Serde<T>(pub T);

// These impl relations are hard to express with generics without accidentally
// including other types, so let's use a macro.
macro_rules! impl_for_regex {
    ($Regex:ty) => {
        impl<'de> Deserialize<'de> for Serde<Option<$Regex>> {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Serde(
                    Option::<Serde<$Regex>>::deserialize(d)?.map(|serde| serde.0),
                ))
            }
        }

        impl<'de> Deserialize<'de> for Serde<$Regex> {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = <Cow<str>>::deserialize(d)?;

                match s.parse() {
                    Ok(regex) => Ok(Serde(regex)),
                    Err(err) => Err(D::Error::custom(err)),
                }
            }
        }

        impl<'a> Serialize for Serde<&'a $Regex> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.as_str().serialize(serializer)
            }
        }

        impl Serialize for Serde<$Regex> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.as_str().serialize(serializer)
            }
        }

        impl<'a> Serialize for Serde<&'a Option<$Regex>> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.as_ref().map(Serde).serialize(serializer)
            }
        }

        impl Serialize for Serde<Option<$Regex>> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.as_ref().map(Serde).serialize(serializer)
            }
        }
    };
}

impl_for_regex!(regex::Regex);
impl_for_regex!(regex::bytes::Regex);

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

#[cfg(test)]
mod tests {
    use super::Serde;
    use serde_json::{from_str, to_string};

    const SAMPLE: &str = r#"[a-z"\]]+\d{1,10}""#;
    const SAMPLE_JSON: &str = r#""[a-z\"\\]]+\\d{1,10}\"""#;

    macro_rules! test_type {
        ($Regex:ty as $name:ident) => {
            mod $name {
                use super::*;

                #[test]
                fn test_serialize() {
                    let re = Serde(<$Regex>::new(SAMPLE).unwrap());
                    assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
                }

                #[test]
                fn test_deserialize() {
                    let deserialized: Serde<$Regex> = from_str(SAMPLE_JSON).unwrap();
                    assert_eq!(deserialized.as_str(), SAMPLE);
                }

                #[test]
                fn test_serialize_some() {
                    let re = Serde(Some(<$Regex>::new(SAMPLE).unwrap()));
                    assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
                }

                #[test]
                fn test_deserialize_some() {
                    let deserialized: Serde<Option<$Regex>> = from_str(SAMPLE_JSON).unwrap();
                    assert_eq!(
                        deserialized.as_ref().map(|regex| regex.as_str()),
                        Some(SAMPLE)
                    );
                }

                #[test]
                fn test_serialize_none() {
                    let re = Serde(None::<$Regex>);
                    assert_eq!(to_string(&re).unwrap(), "null");
                }

                #[test]
                fn test_deserialize_none() {
                    let deserialized: Serde<Option<$Regex>> = from_str("null").unwrap();
                    assert_eq!(deserialized.as_ref().map(|regex| regex.as_str()), None);
                }
            }
        };
    }

    test_type!(::regex::Regex as regex);
    test_type!(::regex::bytes::Regex as bytes_regex);
}
