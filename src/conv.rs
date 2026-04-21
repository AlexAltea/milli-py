use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyList, PyTuple};
use pyo3::IntoPyObjectExt;
use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};

// From https://github.com/mozilla-services/python-canonicaljson-rs/blob/62599b246055a1c8a78e5777acdfe0fd594be3d8/src/lib.rs#L87-L167
#[derive(Debug)]
pub enum PyCanonicalJSONError {
    InvalidConversion { error: String },
    PyErr { error: String },
    DictKeyNotSerializable { typename: String },
    InvalidFloat { value: Py<PyAny> },
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


pub fn to_json<'py>(py: Python<'py>, obj: &Bound<'py, PyAny>) -> Result<serde_json::Value, PyCanonicalJSONError> {
    macro_rules! return_cast {
        ($t:ty, $f:expr) => {
            if let Ok(val) = obj.cast::<$t>() {
                return $f(val);
            }
        };
    }

    macro_rules! return_to_value {
        ($t:ty) => {
            if let Ok(val) = obj.extract::<$t>() {
                return serde_json::value::to_value(val).map_err(|error| {
                    PyCanonicalJSONError::InvalidConversion {
                        error: format!("{}", error),
                    }
                });
            }
        };
    }

    if obj.is_none() {
        return Ok(serde_json::Value::Null);
    }

    return_to_value!(String);
    return_to_value!(bool);
    return_to_value!(u64);
    return_to_value!(i64);

    return_cast!(PyDict, |x: &Bound<'_, PyDict>| {
        let mut map = serde_json::Map::new();
        for (key_obj, value) in x.iter() {
            let key = if key_obj.is_none() {
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
                        .get_type()
                        .name()?
                        .to_string(),
                })
            };
            map.insert(key?, to_json(py, &value)?);
        }
        Ok(serde_json::Value::Object(map))
    });

    return_cast!(PyList, |x: &Bound<'_, PyList>| Ok(serde_json::Value::Array(
        x.iter().map(|x| to_json(py, &x).unwrap()).collect()
    )));

    return_cast!(PyTuple, |x: &Bound<'_, PyTuple>| Ok(serde_json::Value::Array(
        x.iter().map(|x| to_json(py, &x).unwrap()).collect()
    )));

    return_cast!(PyFloat, |x: &Bound<'_, PyFloat>| {
        match serde_json::Number::from_f64(x.value()) {
            Some(n) => Ok(serde_json::Value::Number(n)),
            None => Err(PyCanonicalJSONError::InvalidFloat {
                value: x.clone().into_any().unbind(),
            }),
        }
    });

    // At this point we can't cast it, set up the error object
    Err(PyCanonicalJSONError::InvalidCast {
        typename: obj.get_type().name()?.to_string(),
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
    type Value = Py<PyAny>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'a> Visitor<'de> for ObkvValue<'a> {
    type Value = Py<PyAny>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.into_py_any(self.py).unwrap())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.into_py_any(self.py).unwrap())
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.into_py_any(self.py).unwrap())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.into_py_any(self.py).unwrap())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(value.into_py_any(self.py).unwrap())
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
        Ok(elements.into_py_any(self.py).unwrap())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de> {
        let mut entries = BTreeMap::new();
        while let Some((key, value)) = map.next_entry_seed(PhantomData::<String>, self)? {
            entries.insert(key, value);
        }
        Ok(entries.into_py_any(self.py).unwrap())
    }
}
