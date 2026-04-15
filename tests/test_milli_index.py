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
    
    # Document addition from iterable types
    with tempfile.TemporaryDirectory() as tmp:
        index = milli.Index(tmp)
        # Test tuples
        result = index.add_documents((
            { "id": 0, "title": "Hello world" },
            { "id": 1, "title": "Hello moon" },
        ))
        assert(result.indexed_documents == 2)
        # Test dict_values
        result = index.add_documents({
            2: { "id": 2, "title": "Hello sun" },
            3: { "id": 3, "title": "Hello mars" },
        }.values())
        assert(result.indexed_documents == 2)
        # Test generators
        result = index.add_documents(
            { "id": i, "title": f"Test {i}" } for i in (4, 5)
        )
        assert(result.indexed_documents == 2)
        # Test iterable type with non-dict items
        try:
            index.add_documents([42])
        except TypeError:
            pass
        else:
            assert False, "expected TypeError"
        # Ensure correct items are placed in the index
        assert(index.get_documents([0, 1, 2, 3, 4, 5]) == [
            { "id": 0, "title": "Hello world" },
            { "id": 1, "title": "Hello moon" },
            { "id": 2, "title": "Hello sun" },
            { "id": 3, "title": "Hello mars" },
            { "id": 4, "title": "Test 4" },
            { "id": 5, "title": "Test 5" },
        ])
        del(index)
