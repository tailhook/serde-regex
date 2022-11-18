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

use regex::{Regex, RegexSet, bytes};
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    ops::{Deref, DerefMut}
};

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{Error, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq}
};

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `Regex`
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Serde<T>(pub T);

struct RegexVecVisitor;
struct BytesRegexVecVisitor;

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

impl<'a> Visitor<'a> for BytesRegexVecVisitor {
    type Value = Serde<Vec<bytes::Regex>>;

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


struct RegexHashMapVisitor<K, S>(PhantomData<(K, S)>);
struct BytesRegexHashMapVisitor<K, S>(PhantomData<(K, S)>);

impl<K, S> Default for RegexHashMapVisitor<K, S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, S> Default for BytesRegexHashMapVisitor<K, S> {
    fn default() -> Self {
        Self(Default::default())
    }
}


impl<'a, K, S> Visitor<'a> for RegexHashMapVisitor<K, S>
where
    K: Hash + Eq + Deserialize<'a>,
    S: BuildHasher + Default,
{
    type Value = Serde<HashMap<K, Regex, S>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid map")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'a>
    {
        let mut hashmap = match map.size_hint() {
            Some(size) => HashMap::with_capacity_and_hasher(size, S::default()),
            None => HashMap::with_hasher(S::default()),
        };
        while let Some((key, Serde(value))) = map.next_entry()? {
            hashmap.insert(key, value);
        }
        return Ok(Serde(hashmap));
    }
}

impl<'a, K, S> Visitor<'a> for BytesRegexHashMapVisitor<K, S>
where
    K: Hash + Eq + Deserialize<'a>,
    S: BuildHasher + Default,
{
    type Value = Serde<HashMap<K, bytes::Regex, S>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid map")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'a>
    {
        let mut hashmap = match map.size_hint() {
            Some(size) => HashMap::with_capacity_and_hasher(size, S::default()),
            None => HashMap::with_hasher(S::default()),
        };
        while let Some((key, Serde(value))) = map.next_entry()? {
            hashmap.insert(key, value);
        }
        return Ok(Serde(hashmap));
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
        let s = <Cow<str>>::deserialize(d)?;

        match s.parse() {
            Ok(regex) => Ok(Serde(regex)),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Option<RegexSet>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<RegexSet>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<RegexSet>>::deserialize(d)? {
            Some(Serde(regexset)) => Ok(Serde(Some(regexset))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<RegexSet> {
    fn deserialize<D>(d: D) -> Result<Serde<RegexSet>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let regexes = <Vec<Cow<str>>>::deserialize(d)?;

        match RegexSet::new(regexes) {
            Ok(regexset) => Ok(Serde(regexset)),
            Err(err) => Err(D::Error::custom(err)),
        }
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

impl<'de, K, S> Deserialize<'de> for Serde<HashMap<K, Regex, S>>
where
    K: Hash + Eq + Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_map(RegexHashMapVisitor::default())
    }
}

impl<'de> Deserialize<'de> for Serde<Option<Vec<bytes::Regex>>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<Vec<bytes::Regex>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
         match Option::<Serde<Vec<bytes::Regex>>>::deserialize(d)? {
            Some(Serde(regex)) => Ok(Serde(Some(regex))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de, K, S> Deserialize<'de> for Serde<Option<HashMap<K, bytes::Regex, S>>>
where
    K: Hash + Eq + Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
         match Option::<Serde<HashMap<K, bytes::Regex, S>>>::deserialize(d)? {
            Some(Serde(map)) => Ok(Serde(Some(map))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Option<Vec<Regex>>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<Vec<Regex>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
         match Option::<Serde<Vec<Regex>>>::deserialize(d)? {
            Some(Serde(regex)) => Ok(Serde(Some(regex))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de, K, S> Deserialize<'de> for Serde<Option<HashMap<K, Regex, S>>>
where
    K: Hash + Eq + Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
         match Option::<Serde<HashMap<K, Regex, S>>>::deserialize(d)? {
            Some(Serde(map)) => Ok(Serde(Some(map))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Option<bytes::Regex>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<bytes::Regex>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<bytes::Regex>>::deserialize(d)? {
            Some(Serde(regex)) => Ok(Serde(Some(regex))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<bytes::Regex> {
    fn deserialize<D>(d: D) -> Result<Serde<bytes::Regex>, D::Error>
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

impl<'de> Deserialize<'de> for Serde<bytes::RegexSet> {
    fn deserialize<D>(d: D) -> Result<Serde<bytes::RegexSet>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let regexes = <Vec<Cow<str>>>::deserialize(d)?;

        match bytes::RegexSet::new(regexes) {
            Ok(regexset) => Ok(Serde(regexset)),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Vec<bytes::Regex>> {
    fn deserialize<D>(d: D) -> Result<Serde<Vec<bytes::Regex>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_seq(BytesRegexVecVisitor)
    }
}
impl<'de, K, S> Deserialize<'de> for Serde<HashMap<K, bytes::Regex, S>>
where
    K: Hash + Eq + Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_map(BytesRegexHashMapVisitor::default())
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

/// Serialize function, see crate docs to see how to use it
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

impl Serialize for Serde<Option<RegexSet>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            Some(ref value) => value.patterns().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

impl Serialize for Serde<RegexSet> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.patterns().serialize(serializer)
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

impl<K, S> Serialize for Serde<HashMap<K, Regex, S>>
where
    K: Hash + Eq + Serialize,
    S: BuildHasher + Default,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
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

impl<'a> Serialize for Serde<&'a Option<Vec<Regex>>> {
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

impl<'a, K, S> Serialize for Serde<&'a HashMap<K, Regex, S>>
where
    K: Hash + Eq + Serialize,
    S: BuildHasher + Default,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self.0.iter() {
            map.serialize_entry(key, &Serde(value))?;
        }
        map.end()
    }
}

impl<'a> Serialize for Serde<&'a bytes::Regex> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl Serialize for Serde<bytes::Regex> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl<'a> Serialize for Serde<&'a Option<bytes::Regex>> {
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

impl Serialize for Serde<Option<bytes::Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl Serialize for Serde<bytes::RegexSet> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.patterns().serialize(serializer)
    }
}

impl Serialize for Serde<Vec<bytes::Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl<'a> Serialize for Serde<&'a Option<Vec<bytes::Regex>>> {
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

impl<K, S> Serialize for Serde<HashMap<K, bytes::Regex, S>>
where
    K: Hash + Eq + Serialize,
    S: BuildHasher + Default,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl<'a> Serialize for Serde<&'a Vec<bytes::Regex>> {
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

impl<'a, K, S> Serialize for Serde<&'a HashMap<K, bytes::Regex, S>>
where
    K: Hash + Eq + Serialize,
    S: BuildHasher + Default,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self.0.iter() {
            map.serialize_entry(key, &Serde(value))?;
        }
        map.end()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use serde_json::{json, from_value, from_str, to_string, to_value};
    use regex::{Regex, RegexSet, bytes};
    use crate::Serde;

    const SAMPLE: &str = r#"[a-z"\]]+\d{1,10}""#;
    const SAMPLE_JSON: &str = r#""[a-z\"\\]]+\\d{1,10}\"""#;

    #[test]
    fn test_set() -> Result<(), Box<dyn std::error::Error>> {
        let regexes = &[
            "my(regex)?",
            "other[regex]+",
        ];
        let json = json!(regexes);
        let set: Serde<RegexSet> = from_value(json.clone())?;
        assert_eq!(set.patterns(), regexes);
        assert_eq!(
            to_value(set).expect("serialization error"),
            json
        );
        Ok(())
    }

    #[test]
    fn test_set_option_some() -> Result<(), Box<dyn std::error::Error>> {
        let regexes = &[
            "my(regex)?",
            "other[regex]+",
        ];
        let json = json!(regexes);
        let set: Serde<Option<RegexSet>> = from_value(json.clone())?;
        assert_eq!(set.as_ref().unwrap().patterns(), regexes);
        assert_eq!(
            to_value(set).expect("serialization error"),
            json
        );
        Ok(())
    }

    #[test]
    fn test_set_option_none() -> Result<(), Box<dyn std::error::Error>> {
        let set: Serde<Option<RegexSet>> = from_str("null").unwrap();
        assert!(set.is_none());
        assert_eq!(to_string(&set).unwrap(), "null");
        Ok(())
    }

    #[test]
    fn test_vec() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!(["a.*b", "c?d"]);
        let vec: Serde<Vec<Regex>> = from_value(json)?;
        assert!(vec.0[0].as_str() == "a.*b");
        assert!(vec.0[1].as_str() == "c?d");
        assert!(vec.len() == 2);
        Ok(())
    }

    #[test]
    fn test_hashmap() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!({"a": "a.*b", "b": "c?d"});
        let map: Serde<HashMap<String, Regex>> = from_value(json)?;
        assert!(map.0["a"].as_str() == "a.*b");
        assert!(map.0["b"].as_str() == "c?d");
        assert!(map.len() == 2);
        Ok(())
    }

    #[test]
    fn test_simple() {
        let re: Serde<Regex> = from_str(SAMPLE_JSON).unwrap();
        assert_eq!(re.as_str(), SAMPLE);
        assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
    }

    #[test]
    fn test_option_some() {
        let re: Serde<Option<Regex>> = from_str(SAMPLE_JSON).unwrap();
        assert_eq!(re.as_ref().map(|regex| regex.as_str()), Some(SAMPLE));
        assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
    }

    #[test]
    fn test_option_none() {
        let re: Serde<Option<Regex>> = from_str("null").unwrap();
        assert!(re.is_none());
        assert_eq!(to_string(&re).unwrap(), "null");
    }

    #[test]
    fn test_set_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let regexes = &[
            "regex.*test",
            "test( )??regex+",
        ];
        let json = json!(regexes);
        let set: Serde<bytes::RegexSet> = from_value(json.clone())?;
        assert_eq!(set.patterns(), regexes);
        assert_eq!(
            to_value(set).expect("serialization error"),
            json
        );
        Ok(())
    }

    #[test]
    fn test_vec_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!(["a.*b", "c?d"]);
        let vec: Serde<Vec<bytes::Regex>> = from_value(json)?;
        assert!(vec.0[0].as_str() == "a.*b");
        assert!(vec.0[1].as_str() == "c?d");
        assert!(vec.len() == 2);
        Ok(())
    }

    #[test]
    fn test_hashmap_bytes() -> Result<(), Box<dyn std::error::Error>> {
        // let json = json!(["a.*b", "c?d"]);
        let json = json!({ "c": "a.*b", "d": "c?d" });
        let map: Serde<HashMap<String, bytes::Regex>> = from_value(json)?;
        assert!(map.0["c"].as_str() == "a.*b");
        assert!(map.0["d"].as_str() == "c?d");
        assert!(map.len() == 2);
        Ok(())
    }

    #[test]
    fn test_option_vec() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!(["a.*b", "c?d"]);
        let vec: Serde<Option<Vec<Regex>>> = from_value(json)?;
        assert!(vec.is_some());
        let v = vec.0.unwrap();
        assert!(v[0].as_str() == "a.*b");
        assert!(v[1].as_str() == "c?d");
        assert!(v.len() == 2);
        Ok(())
    }
    #[test]
    fn test_option_hashmap() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!({"a": "a.*b", "b": "c?d"});
        let map: Serde<Option<HashMap<String, Regex>>> = from_value(json)?;
        assert!(map.is_some());
        let v = map.0.unwrap();
        assert!(v["a"].as_str() == "a.*b");
        assert!(v["b"].as_str() == "c?d");
        assert!(v.len() == 2);
        Ok(())
    }
    #[test]
    fn test_option_vec_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!(["a.*b", "c?d"]);
        let vec: Serde<Option<Vec<bytes::Regex>>> = from_value(json)?;
        assert!(vec.is_some());
        let v = vec.0.unwrap();
        assert!(v[0].as_str() == "a.*b");
        assert!(v[1].as_str() == "c?d");
        assert!(v.len() == 2);
        Ok(())
    }
    #[test]
    fn test_option_hashamp_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!({"a": "a.*b", "b": "c?d"});
        let map: Serde<Option<HashMap<String, bytes::Regex>>> = from_value(json)?;
        assert!(map.is_some());
        let v = map.0.unwrap();
        assert!(v["a"].as_str() == "a.*b");
        assert!(v["b"].as_str() == "c?d");
        assert!(v.len() == 2);
        Ok(())
    }
    #[test]
    fn test_option_vec_none() -> Result<(), Box<dyn std::error::Error>> {
        let vec: Serde<Option<Vec<bytes::Regex>>> = from_str("null")?;
        assert!(vec.is_none());
        Ok(())
    }
    #[test]
    fn test_option_hashmap_none() -> Result<(), Box<dyn std::error::Error>> {
        let map: Serde<Option<HashMap<String, bytes::Regex>>> = from_str("null")?;
        assert!(map.is_none());
        Ok(())
    }

    #[test]
    fn test_bytes_simple() {
        let re: Serde<bytes::Regex> = from_str(SAMPLE_JSON).unwrap();
        assert_eq!(re.as_str(), SAMPLE);
        assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
    }

    #[test]
    fn test_bytes_option_some() {
        let re: Serde<Option<bytes::Regex>> = from_str(SAMPLE_JSON).unwrap();
        assert_eq!(re.as_ref().map(|regex| regex.as_str()), Some(SAMPLE));
        assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
    }

    #[test]
    fn test_bytes_option_none() {
        let re: Serde<Option<bytes::Regex>> = from_str("null").unwrap();
        assert!(re.is_none());
        assert_eq!(to_string(&re).unwrap(), "null");
    }
}
