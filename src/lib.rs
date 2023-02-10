#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate milli as mi;

use pyo3::prelude::*;
use pyo3::types::*;

use mi::Index;
use mi::update::IndexerConfig;
use mi::update::IndexDocumentsConfig;
use mi::update::IndexDocuments;
use mi::Search;
use mi::SearchResult;
use mi::DocumentId;
mod conv;
mod obkv;

use serde::de::{DeserializeSeed};

use mi::documents::*;
/// Macro used to generate documents, with the same syntax as `serde_json::json`
pub fn objects_from_json_value(json: serde_json::Value) -> Vec<mi::Object> {
    let documents = match json {
        object @ serde_json::Value::Object(_) => vec![object],
        serde_json::Value::Array(objects) => objects,
        invalid => {
            panic!("an array of objects must be specified, {:#?} is not an array", invalid)
        }
    };
    let mut objects = vec![];
    for document in documents {
        let object = match document {
            serde_json::Value::Object(object) => object,
            invalid => panic!("an object must be specified, {:#?} is not an object", invalid),
        };
        objects.push(object);
    }
    objects
}
pub fn documents_batch_reader_from_objects(
    objects: impl IntoIterator<Item = mi::Object>,
) -> DocumentsBatchReader<std::io::Cursor<Vec<u8>>> {
    let mut builder = DocumentsBatchBuilder::new(Vec::new());
    for object in objects {
        builder.append_json_object(&object).unwrap();
    }
    let vector = builder.into_inner().unwrap();
    DocumentsBatchReader::from_reader(std::io::Cursor::new(vector)).unwrap()
}

#[pyclass(name="Index")]
struct PyIndex {
    index: Index,
}

#[pymethods]
impl PyIndex {
    #[new]
    fn new(path: String, map_size: usize) -> Self {
        let mut options = mi::heed::EnvOpenOptions::new();
        options.map_size(map_size);
        let index = Index::new(options, &path).unwrap();
        return PyIndex{ index };
    }

    fn add_documents(&self, py: Python<'_>, obj: PyObject) -> PyResult<DocumentAdditionResult> {
        let mut wtxn = self.index.write_txn().unwrap();
        let config = IndexerConfig::default();
        let indexing_config = IndexDocumentsConfig::default();
        let mut builder = IndexDocuments::new(
            &mut wtxn, &self.index, &config, indexing_config.clone(), |_| (), || false).unwrap();

        let documents = conv::to_json(py, &obj).unwrap();
        let documents = objects_from_json_value(documents);
        let reader = documents_batch_reader_from_objects(documents);

        let (builder, user_error) = builder.add_documents(reader).unwrap();
        println!("user_error = {:?}", user_error);
        builder.execute().unwrap();
        wtxn.commit().unwrap();
        Ok(DocumentAdditionResult{})
    }

    fn get_document(&self, py: Python<'_>, id: DocumentId) -> PyResult<Py<PyDict>> {
        let mut rtxn = self.index.read_txn().unwrap();
        let (docid, obkv) = self.index.documents(&rtxn, [id]).unwrap()[0];
        let fields = self.index.fields_ids_map(&rtxn).unwrap();

        // Deserialize JSON into Python objects
        let dict = PyDict::new(py);
        for (id, bytes) in obkv.iter() {
            let key = fields.name(id);
            let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
            let value = obkv::ObkvValue::new(py).deserialize(&mut deserializer).unwrap();
            dict.set_item(key, value).unwrap();
        }
        Ok(dict.into())
    }

    fn search(&self, query: String) -> Vec<DocumentId> {
        let mut rtxn = self.index.read_txn().unwrap();
        let mut search = Search::new(&rtxn, &self.index);
        search.query(query);
        let results = search.execute().unwrap();
        return results.documents_ids;
    }
}

#[pyclass]
pub struct DocumentAdditionResult {
}

#[pymodule]
fn milli(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyIndex>()?;
    Ok(())
}
