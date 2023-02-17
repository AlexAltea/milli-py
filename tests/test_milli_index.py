import milli
import tempfile

def test_milli_index():
    # Index search
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

    # Index update
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
        assert(index.get_documents([3, 4, 5]) == [
            {'id': 1, 'title': 'Hello mars'},
            {'id': 2, 'title': 'Hello helios', 'content': 'This is yet another sample'},
            {'id': 0, 'title': 'Hello world', 'content': 'This is a sample', 'people': True},
        ])
        del(index)
