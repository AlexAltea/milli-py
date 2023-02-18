extern crate milli as mi;

use std::ops::Deref;

use pyo3::prelude::*;
use pyo3::types::*;

use mi::{DocumentId, Index, Search};
use mi::documents::{DocumentsBatchBuilder, DocumentsBatchReader};
use mi::update::{DeleteDocuments, DocumentAdditionResult, DocumentDeletionResult,
    IndexerConfig, IndexDocumentsConfig, IndexDocumentsMethod, IndexDocuments};
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
    fn new(path: String, map_size: Option<usize>) -> Self {
        let mut options = mi::heed::EnvOpenOptions::new();
        if map_size.is_some() {
            options.map_size(map_size.unwrap());
        }
        let index = Index::new(options, &path).unwrap();
        return PyIndex{ index };
    }

    fn add_documents(&self, py: Python<'_>, list: &PyList, update_method: Option<PyIndexDocumentsMethod>) -> PyResult<PyDocumentAdditionResult> {
        let mut config = IndexDocumentsConfig::default();
        if update_method.is_some() {
            config.update_method = update_method.unwrap().into();
        }

        let mut wtxn = self.write_txn().unwrap();
        let indexer_config = IndexerConfig::default();
        let builder = IndexDocuments::new(
            &mut wtxn, &self, &indexer_config, config.clone(), |_| (), || false).unwrap();

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
        let result = builder.execute().unwrap();
        wtxn.commit().unwrap();
        Ok(result.into())
    }

    fn delete_documents(&self, ids: Vec<DocumentId>) -> PyResult<PyDocumentDeletionResult> {
        let mut wtxn = self.write_txn().unwrap();
        let mut builder = DeleteDocuments::new(&mut wtxn, self).unwrap();
        for docid in ids {
            builder.delete_document(docid);
        }
        let result = builder.execute().unwrap();
        wtxn.commit().unwrap();
        Ok(result.into())
    }

    fn get_document(&self, py: Python<'_>, id: DocumentId) -> PyResult<Py<PyDict>> {
        let rtxn = self.read_txn().unwrap();
        let (_docid, obkv) = self.index.documents(&rtxn, [id]).unwrap()[0];
        let dict = obkv_to_pydict!(self, py, rtxn, obkv);
        Ok(dict.into())
    }

    fn get_documents(&self, py: Python<'_>, ids: Vec<DocumentId>) -> PyResult<Py<PyList>> {
        let rtxn = self.read_txn().unwrap();
        let docs = self.documents(&rtxn, ids).unwrap();
        let list = PyList::empty(py);
        for (_docid, obkv) in docs {
            list.append(obkv_to_pydict!(self, py, rtxn, obkv)).unwrap();
        }
        Ok(list.into())
    }

    fn search(&self, query: String) -> Vec<DocumentId> {
        let rtxn = self.read_txn().unwrap();
        let mut search = Search::new(&rtxn, &self);
        search.query(query);
        let results = search.execute().unwrap();
        return results.documents_ids;
    }
}

impl Deref for PyIndex {
    type Target = Index;
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl Drop for PyIndex {
    fn drop(&mut self) {
        self.index.clone().prepare_for_closing();
    }
}

#[derive(Clone)]
#[pyclass(name="IndexDocumentsMethod")]
enum PyIndexDocumentsMethod {
    ReplaceDocuments,
    UpdateDocuments,
}
impl From<PyIndexDocumentsMethod> for IndexDocumentsMethod {
    fn from(value: PyIndexDocumentsMethod) -> Self {
        match value {
            PyIndexDocumentsMethod::ReplaceDocuments => Self::ReplaceDocuments,
            PyIndexDocumentsMethod::UpdateDocuments => Self::UpdateDocuments,
        }
    }
}

#[pyclass(name="DocumentAdditionResult")]
struct PyDocumentAdditionResult {
    #[pyo3(get, set)]
    indexed_documents: u64,
    #[pyo3(get, set)]
    number_of_documents: u64,
}
impl From<DocumentAdditionResult> for PyDocumentAdditionResult {
    fn from(value: DocumentAdditionResult) -> Self {
        PyDocumentAdditionResult{
            indexed_documents: value.indexed_documents,
            number_of_documents: value.number_of_documents,
        }
    }
}

#[pyclass(name="DocumentDeletionResult")]
struct PyDocumentDeletionResult {
    #[pyo3(get, set)]
    deleted_documents: u64,
    #[pyo3(get, set)]
    remaining_documents: u64,
}
impl From<DocumentDeletionResult> for PyDocumentDeletionResult {
    fn from(value: DocumentDeletionResult) -> Self {
        PyDocumentDeletionResult{
            deleted_documents: value.deleted_documents,
            remaining_documents: value.remaining_documents,
        }
    }
}

#[pymodule]
fn milli(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyIndex>()?;
    m.add_class::<PyIndexDocumentsMethod>()?;
    m.add_class::<PyDocumentAdditionResult>()?;
    m.add_class::<PyDocumentDeletionResult>()?;
    Ok(())
}
