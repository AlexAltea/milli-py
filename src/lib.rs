extern crate milli as mi;

use pyo3::prelude::*;
use pyo3::types::*;

use mi::{DocumentId, Index, Search};
use mi::documents::{DocumentsBatchBuilder, DocumentsBatchReader};
use mi::update::{IndexerConfig, IndexDocumentsConfig, IndexDocuments};
use serde::Deserializer;

mod conv;

// Helpers
macro_rules! obkv_to_pydict {
    ($self:ident, $py:ident, $rtxn:ident, $obkv:ident) => {{
        let fields = $self.index.fields_ids_map(&$rtxn).unwrap();
        let dict = PyDict::new($py);
        for (id, bytes) in $obkv.iter() {
            let key = fields.name(id);
            let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
            let value = conv::ObkvValue::new($py);
            let value = deserializer.deserialize_any(value).unwrap();
            dict.set_item(key, value).unwrap();
        }
        dict
    }};
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

    fn add_documents(&self, py: Python<'_>, list: &PyList) -> PyResult<DocumentAdditionResult> {
        let mut wtxn = self.index.write_txn().unwrap();
        let config = IndexerConfig::default();
        let indexing_config = IndexDocumentsConfig::default();
        let builder = IndexDocuments::new(
            &mut wtxn, &self.index, &config, indexing_config.clone(), |_| (), || false).unwrap();

        // Convert Python array into Vec<milli::Object>
        let list = list.to_object(py);
        let list = conv::to_json(py, &list)?;
        let mut docbuilder = DocumentsBatchBuilder::new(Vec::new());
        for item in list.as_array().unwrap() {
            let object = item.as_object().unwrap();
            docbuilder.append_json_object(object).unwrap();
        }
        let vector = docbuilder.into_inner().unwrap();
        let reader = DocumentsBatchReader::from_reader(std::io::Cursor::new(vector)).unwrap();

        let (builder, _user_error) = builder.add_documents(reader).unwrap();
        builder.execute().unwrap();
        wtxn.commit().unwrap();
        Ok(DocumentAdditionResult{})
    }

    fn get_document(&self, py: Python<'_>, id: DocumentId) -> PyResult<Py<PyDict>> {
        let rtxn = self.index.read_txn().unwrap();
        let (_docid, obkv) = self.index.documents(&rtxn, [id]).unwrap()[0];
        let dict = obkv_to_pydict!(self, py, rtxn, obkv);
        Ok(dict.into())
    }

    fn get_documents(&self, py: Python<'_>, ids: Vec<DocumentId>) -> PyResult<Py<PyList>> {
        let rtxn = self.index.read_txn().unwrap();
        let docs = self.index.documents(&rtxn, ids).unwrap();
        let list = PyList::empty(py);
        for (_docid, obkv) in docs {
            list.append(obkv_to_pydict!(self, py, rtxn, obkv)).unwrap();
        }
        Ok(list.into())
    }

    fn search(&self, query: String) -> Vec<DocumentId> {
        let rtxn = self.index.read_txn().unwrap();
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
