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
extern crate serde;
extern crate regex;

#[cfg(test)] extern crate serde_json;

use std::fmt;
use regex::Regex;

use serde::de::{Visitor, Error};
use serde::{Deserializer, Serializer};


struct RegexVisitor;


impl<'a> Visitor<'a> for RegexVisitor {
    type Value = Regex;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid regular expression")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: Error
    {
        Regex::new(value).map_err(E::custom)
    }
}

/// Deserialize function, see crate docs to see how to use it
pub fn deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where D: Deserializer<'de>,
{
    deserializer.deserialize_str(RegexVisitor)
}

/// Deserialize function, see crate docs to see how to use it
pub fn serialize<S>(value: &Regex, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
    serializer.serialize_str(value.as_str())
}
