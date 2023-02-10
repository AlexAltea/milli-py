// From https://github.com/mre/hyperjson/blob/87335d442869832b46e7e9f10800a27360dd8169/src/lib.rs#L397
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use pyo3::prelude::*;
use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};

#[derive(Copy, Clone)]
pub struct ObkvValue<'a> {
    py: Python<'a>,
}

impl<'a> ObkvValue<'a> {
    pub fn new(py: Python<'a>) -> ObkvValue<'a> {
        ObkvValue { py }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ObkvValue<'a> {
    type Value = PyObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'a> Visitor<'de> for ObkvValue<'a> {
    type Value = PyObject;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.to_object(self.py))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.to_object(self.py))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.to_object(self.py))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.to_object(self.py))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.to_object(self.py))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(self.py.None())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de> {
        let mut elements = Vec::new();
        while let Some(elem) = seq.next_element_seed(self)? {
            elements.push(elem);
        }
        Ok(elements.to_object(self.py))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de> {
        let mut entries = BTreeMap::new();
        while let Some((key, value)) = map.next_entry_seed(PhantomData::<String>, self)? {
            entries.insert(key, value);
        }
        Ok(entries.to_object(self.py))
    }
}
