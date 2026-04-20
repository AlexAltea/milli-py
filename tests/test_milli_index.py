import milli
import tempfile
import unittest

class TestMilliIndex(unittest.TestCase):
    def test_document_addition(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = milli.Index(tmp)
            self.assertEqual(index.primary_key(), None)

            # Without an explicit external ID nothing gets indexed
            result = index.add_documents([
                { "title": "Hello world" },
            ])
            self.assertEqual(result.indexed_documents, 0)
            self.assertEqual(result.number_of_documents, 0)
            self.assertEqual(index.primary_key(), None)

            # With an explicit external ID
            result = index.add_documents([
                { "id": 1, "title": "Hello world" },
            ])
            self.assertEqual(result.indexed_documents, 1)
            self.assertEqual(result.number_of_documents, 1)
            self.assertEqual(index.primary_key(), "id")

            # Note the document has an external ID (== 1) and internal ID (== 0)
            self.assertEqual(index.get_documents([0]), [{ "id": 1, "title": "Hello world" }])
            del(index)

    def test_document_listing(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = milli.Index(tmp)
            index.add_documents([
                { "id": 11, "title": "Hello moon", "content": "This is another sample" },
                { "id": 12, "title": "Hello sun", "content": "This is yet another sample" },
            ])
            docs = index.all_documents()
            self.assertEqual(next(docs), (0, { "id": 11, "title": "Hello moon", "content": "This is another sample" }))
            self.assertEqual(next(docs), (1, { "id": 12, "title": "Hello sun", "content": "This is yet another sample" }))
            del(index)

    def test_document_search(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = milli.Index(tmp)
            index.add_documents([
                { "id": 0, "title": "Hello world", "content": "This is a sample" },
                { "id": 1, "title": "Hello moon", "content": "This is another sample" },
                { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
            ])
            results = index.search("wrold")
            document = index.get_document(results[0])
            self.assertEqual(document['title'], "Hello world")
            del(index)

    def test_document_update(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = milli.Index(tmp)
            index.add_documents([
                { "id": 0, "title": "Hello world", "content": "This is a sample" },
                { "id": 1, "title": "Hello moon", "content": "This is another sample" },
                { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
            ])
            index.add_documents([
                { "id": 1, "title": "Hello mars" },
            ], milli.IndexDocumentsMethod.ReplaceDocuments)
            index.add_documents([
                { "id": 2, "title": "Hello helios" },
                { "id": 0, "people": True },
            ], milli.IndexDocumentsMethod.UpdateDocuments)
            self.assertEqual(index.get_documents([1, 2, 0]), [
                {'id': 1, 'title': 'Hello mars'},
                {'id': 2, 'title': 'Hello helios', 'content': 'This is yet another sample'},
                {'id': 0, 'title': 'Hello world', 'content': 'This is a sample', 'people': True},
            ])
            del(index)

    def test_document_removal(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = milli.Index(tmp)
            index.add_documents([
                { "id": 0, "title": "Hello world", "content": "This is a sample" },
                { "id": 1, "title": "Hello moon", "content": "This is another sample" },
                { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
            ])
            result = index.delete_documents(["2", "0", "7"])
            self.assertEqual(result, 2)
            self.assertEqual(index.get_document(1)['id'], 1)
            del(index)

    def test_document_clearing(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = milli.Index(tmp)
            index.add_documents([
                { "id": 0, "title": "Hello world", "content": "This is a sample" },
                { "id": 1, "title": "Hello moon", "content": "This is another sample" },
                { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
            ])
            result = index.clear_documents()
            self.assertEqual(result, 3)
            del(index)

    def test_document_addition_from_iterable_types(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = milli.Index(tmp)
            # Test tuples
            result = index.add_documents((
                { "id": 0, "title": "Hello world" },
                { "id": 1, "title": "Hello moon" },
            ))
            self.assertEqual(result.indexed_documents, 2)
            # Test dict_values
            result = index.add_documents({
                2: { "id": 2, "title": "Hello sun" },
                3: { "id": 3, "title": "Hello mars" },
            }.values())
            self.assertEqual(result.indexed_documents, 2)
            # Test generators
            result = index.add_documents(
                { "id": i, "title": f"Test {i}" } for i in (4, 5)
            )
            self.assertEqual(result.indexed_documents, 2)
            # Test iterable type with non-dict items
            with self.assertRaises(TypeError):
                index.add_documents([42])
            # Ensure correct items are placed in the index
            self.assertEqual(index.get_documents([0, 1, 2, 3, 4, 5]), [
                { "id": 0, "title": "Hello world" },
                { "id": 1, "title": "Hello moon" },
                { "id": 2, "title": "Hello sun" },
                { "id": 3, "title": "Hello mars" },
                { "id": 4, "title": "Test 4" },
                { "id": 5, "title": "Test 5" },
            ])
            del(index)
