import milli
import tempfile

def test_milli_index():
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
