use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyList, PyTuple};
use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};

// From https://github.com/mozilla-services/python-canonicaljson-rs/blob/62599b246055a1c8a78e5777acdfe0fd594be3d8/src/lib.rs#L87-L167
#[derive(Debug)]
pub enum PyCanonicalJSONError {
    InvalidConversion { error: String },
    PyErr { error: String },
    DictKeyNotSerializable { typename: String },
    InvalidFloat { value: PyObject },
    InvalidCast { typename: String },
}

impl From<pyo3::PyErr> for PyCanonicalJSONError {
    fn from(error: pyo3::PyErr) -> PyCanonicalJSONError {
        PyCanonicalJSONError::PyErr {
            error: format!("{:?}", error),
        }
    }
}

impl From<PyCanonicalJSONError> for pyo3::PyErr {
    fn from(e: PyCanonicalJSONError) -> pyo3::PyErr {
        match e {
            PyCanonicalJSONError::InvalidConversion { error } => {
                PyErr::new::<PyTypeError, _>(format!("Conversion error: {:?}", error))
            }
            PyCanonicalJSONError::PyErr { error } => {
                PyErr::new::<PyTypeError, _>(format!("Python Runtime exception: {}", error))
            }
            PyCanonicalJSONError::DictKeyNotSerializable { typename } => {
                PyErr::new::<PyTypeError, _>(format!(
                    "Dictionary key is not serializable: {}",
                    typename
                ))
            }
            PyCanonicalJSONError::InvalidFloat { value } => {
                PyErr::new::<PyTypeError, _>(format!("Invalid float: {:?}", value))
            }
            PyCanonicalJSONError::InvalidCast { typename } => {
                PyErr::new::<PyTypeError, _>(format!("Invalid type: {}", typename))
            }
        }
    }
}


pub fn to_json(py: Python, obj: &PyObject) -> Result<serde_json::Value, PyCanonicalJSONError> {
    macro_rules! return_cast {
        ($t:ty, $f:expr) => {
            if let Ok(val) = obj.downcast::<$t>(py) {
                return $f(val);
            }
        };
    }

    macro_rules! return_to_value {
        ($t:ty) => {
            if let Ok(val) = obj.extract::<$t>(py) {
                return serde_json::value::to_value(val).map_err(|error| {
                    PyCanonicalJSONError::InvalidConversion {
                        error: format!("{}", error),
                    }
                });
            }
        };
    }

    if obj.as_ref(py).eq(&py.None())? {
        return Ok(serde_json::Value::Null);
    }

    return_to_value!(String);
    return_to_value!(bool);
    return_to_value!(u64);
    return_to_value!(i64);

    return_cast!(PyDict, |x: &PyDict| {
        let mut map = serde_json::Map::new();
        for (key_obj, value) in x.iter() {
            let key = if key_obj.eq(py.None().as_ref(py))? {
                Ok("null".to_string())
            } else if let Ok(val) = key_obj.extract::<bool>() {
                Ok(if val {
                    "true".to_string()
                } else {
                    "false".to_string()
                })
            } else if let Ok(val) = key_obj.str() {
                Ok(val.to_string())
            } else {
                Err(PyCanonicalJSONError::DictKeyNotSerializable {
                    typename: key_obj
                        .to_object(py)
                        .as_ref(py)
                        .get_type()
                        .name()?
                        .to_string(),
                })
            };
            map.insert(key?, to_json(py, &value.to_object(py))?);
        }
        Ok(serde_json::Value::Object(map))
    });

    return_cast!(PyList, |x: &PyList| Ok(serde_json::Value::Array(
        x.iter().map(|x| to_json(py, &x.to_object(py)).unwrap()).collect()
    )));

    return_cast!(PyTuple, |x: &PyTuple| Ok(serde_json::Value::Array(
        x.iter().map(|x| to_json(py, &x.to_object(py)).unwrap()).collect()
    )));

    return_cast!(PyFloat, |x: &PyFloat| {
        match serde_json::Number::from_f64(x.value()) {
            Some(n) => Ok(serde_json::Value::Number(n)),
            None => Err(PyCanonicalJSONError::InvalidFloat {
                value: x.to_object(py),
            }),
        }
    });

    // At this point we can't cast it, set up the error object
    Err(PyCanonicalJSONError::InvalidCast {
        typename: obj.as_ref(py).get_type().name()?.to_string(),
    })
}


// From https://github.com/mre/hyperjson/blob/87335d442869832b46e7e9f10800a27360dd8169/src/lib.rs#L397
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
