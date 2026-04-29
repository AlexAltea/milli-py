extern crate milli as mi;

use std::ops::Deref;
use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::*;

use mi::{CreateOrOpen, DocumentId, Index, Search};
use mi::documents::{DocumentsBatchBuilder, DocumentsBatchReader, PrimaryKey};
use mi::progress::{EmbedderStats, Progress};
use mi::update::{ClearDocuments, DocumentAdditionResult,
    IndexerConfig, IndexDocumentsConfig, IndexDocumentsMethod, IndexDocuments};
use mi::update::new::indexer;
use mi::update::new::indexer::DocumentDeletion;
use mi::ThreadPoolNoAbortBuilder;
use mi::vector::RuntimeEmbedders;
use http_client::policy::IpPolicy;
use bumpalo::Bump;
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
    #[pyo3(signature = (path, map_size=None))]
    fn new(path: String, map_size: Option<usize>) -> Self {
        let mut options = mi::heed::EnvOpenOptions::new().read_txn_without_tls();
        if map_size.is_some() {
            options.map_size(map_size.unwrap());
        }
        let index = Index::new(options, &path, CreateOrOpen::create_without_shards()).unwrap();
        return PyIndex{ index };
    }

    #[pyo3(signature = (documents, update_method=None))]
    fn add_documents(&self, py: Python<'_>, documents: &Bound<'_, PyAny>, update_method: Option<PyIndexDocumentsMethod>) -> PyResult<PyDocumentAdditionResult> {
        let mut config = IndexDocumentsConfig::default();
        if update_method.is_some() {
            config.update_method = update_method.unwrap().into();
        }

        let mut wtxn = self.write_txn().unwrap();
        let indexer_config = IndexerConfig::default();
        let embedder_stats: Arc<EmbedderStats> = Default::default();
        let ip_policy = IpPolicy::danger_always_allow();
        let builder = IndexDocuments::new(
            &mut wtxn, &self, &indexer_config, config.clone(), |_| (), || false,
            &embedder_stats, &ip_policy).unwrap();

        // Convert Python iterable types into Vec<milli::Object>
        let mut docbuilder = DocumentsBatchBuilder::new(Vec::new());
        for item in documents.try_iter()? {
            let item = item?;
            let value = conv::to_json(py, &item)?;
            let object = value.as_object().ok_or_else(|| {
                pyo3::exceptions::PyTypeError::new_err("each document must be a dict")
            })?;
            docbuilder.append_json_object(object).unwrap();
        }
        let vector = docbuilder.into_inner().unwrap();
        let reader = DocumentsBatchReader::from_reader(std::io::Cursor::new(vector)).unwrap();

        let (builder, _user_error) = builder.add_documents(reader).unwrap();
        let result = builder.execute().unwrap();
        wtxn.commit().unwrap();
        Ok(result.into())
    }

    fn all_documents(&self, py: Python<'_>) -> PyResult<Py<PyIterator>> {
        let rtxn = self.read_txn().unwrap();
        let docs = self.index.all_documents(&rtxn).unwrap();

        // TODO: Wrap as a Python iterator without converting to list
        let list = PyList::empty(py);
        for document in docs {
            let (docid, obkv) = document.unwrap();
            let doc = obkv_to_pydict!(self, py, rtxn, obkv);
            list.append((docid, doc)).unwrap();
        }
        let iter = PyIterator::from_object(&list).unwrap();
        Ok(iter.into())
    }

    fn clear_documents(&self) -> PyResult<u64> {
        let mut wtxn = self.write_txn().unwrap();
        let builder = ClearDocuments::new(&mut wtxn, self);
        let result = builder.execute().unwrap();
        wtxn.commit().unwrap();
        Ok(result.into())
    }

    fn delete_documents(&self, ids: Vec<DocumentId>) -> PyResult<u64> {
        let indexer_config = IndexerConfig::default();
        let pool = &indexer_config.thread_pool;

        let mut wtxn = self.write_txn().unwrap();
        let rtxn = self.read_txn().unwrap();
        let db_fields_ids_map = self.index.fields_ids_map(&rtxn).unwrap();
        let new_fields_ids_map = db_fields_ids_map.clone();

        // Keep only IDs that actually exist in the index
        let existing = self.index.documents_ids(&rtxn).unwrap();
        let to_delete: Vec<DocumentId> = ids.into_iter().filter(|id| existing.contains(*id)).collect();
        let deleted = to_delete.len() as u64;

        let mut deletion = DocumentDeletion::new();
        deletion.delete_documents_by_docids(to_delete.into_iter().collect());

        let indexer_alloc = Bump::new();
        let pk_name = self.index.primary_key(&rtxn).unwrap().unwrap();
        let primary_key = PrimaryKey::new(pk_name, &db_fields_ids_map).unwrap();
        let document_changes = deletion.into_changes(&indexer_alloc, primary_key);

        pool.install(|| {
            indexer::index(
                &mut wtxn,
                &self.index,
                &ThreadPoolNoAbortBuilder::new().build().unwrap(),
                indexer_config.grenad_parameters(),
                &db_fields_ids_map,
                new_fields_ids_map,
                None,
                &document_changes,
                RuntimeEmbedders::default(),
                &|| false,
                &Progress::default(),
                &IpPolicy::danger_always_allow(),
                &Default::default(),
            )
        }).unwrap().unwrap();

        wtxn.commit().unwrap();
        Ok(deleted)
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
        Ok(list.unbind())
    }

    fn primary_key(&self) -> PyResult<Option<String>> {
        let rtxn = self.read_txn().unwrap();
        let result = self.index.primary_key(&rtxn).unwrap();
        let converted_result = result.map(|s| s.to_string());
        Ok(converted_result)
    }

    fn search(&self, query: String) -> Vec<DocumentId> {
        let rtxn = self.read_txn().unwrap();
        let progress = Progress::default();
        let mut search = Search::new(&rtxn, &self, &progress);
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
#[pyclass(name="IndexDocumentsMethod", from_py_object)]
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

#[pymodule]
fn milli(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIndex>()?;
    m.add_class::<PyIndexDocumentsMethod>()?;
    m.add_class::<PyDocumentAdditionResult>()?;
    Ok(())
}
