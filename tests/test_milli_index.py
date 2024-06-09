import milli
import tempfile

def test_milli_index():
    # Document addition and primary keys
    with tempfile.TemporaryDirectory() as tmp:
        index = milli.Index(tmp)
        assert(index.primary_key() == None)

        # Without an explicit external ID nothing gets indexed
        result = index.add_documents([
            { "title": "Hello world" },
        ])
        assert(result.indexed_documents == 0)
        assert(result.number_of_documents == 0)
        assert(index.primary_key() == None)

        # With an explicit external ID
        result = index.add_documents([
            { "id": 1, "title": "Hello world" },
        ])
        assert(result.indexed_documents == 1)
        assert(result.number_of_documents == 1)
        assert(index.primary_key() == "id")

        # Note the document has an external ID (== 1) and internal ID (== 0)
        assert(index.get_documents([0]) == [{ "id": 1, "title": "Hello world" }])
        del(index)

    # Document listing
    with tempfile.TemporaryDirectory() as tmp:
        index = milli.Index(tmp)
        index.add_documents([
            { "id": 11, "title": "Hello moon", "content": "This is another sample" },
            { "id": 12, "title": "Hello sun", "content": "This is yet another sample" },
        ])
        docs = index.all_documents()
        assert(next(docs) == (0, { "id": 11, "title": "Hello moon", "content": "This is another sample" }))
        assert(next(docs) == (1, { "id": 12, "title": "Hello sun", "content": "This is yet another sample" }))
        del(index)

    # Document search
    with tempfile.TemporaryDirectory() as tmp:
        index = milli.Index(tmp)
        index.add_documents([
            { "id": 0, "title": "Hello world", "content": "This is a sample" },
            { "id": 1, "title": "Hello moon", "content": "This is another sample" },
            { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
        ])
        results = index.search("wrold")
        document = index.get_document(results[0])
        assert(document['title'] == "Hello world")
        del(index)

    # Document update
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
        assert(index.get_documents([1, 2, 0]) == [
            {'id': 1, 'title': 'Hello mars'},
            {'id': 2, 'title': 'Hello helios', 'content': 'This is yet another sample'},
            {'id': 0, 'title': 'Hello world', 'content': 'This is a sample', 'people': True},
        ])
        del(index)

    # Document removal
    with tempfile.TemporaryDirectory() as tmp:
        index = milli.Index(tmp)
        index.add_documents([
            { "id": 0, "title": "Hello world", "content": "This is a sample" },
            { "id": 1, "title": "Hello moon", "content": "This is another sample" },
            { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
        ])
        result = index.delete_documents(["2", "0", "7"])
        assert(result == 2)
        assert(index.get_document(1)['id'] == 1)
        del(index)

    # Document clearing
    with tempfile.TemporaryDirectory() as tmp:
        index = milli.Index(tmp)
        index.add_documents([
            { "id": 0, "title": "Hello world", "content": "This is a sample" },
            { "id": 1, "title": "Hello moon", "content": "This is another sample" },
            { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
        ])
        result = index.clear_documents()
        assert(result == 3)
        del(index)
