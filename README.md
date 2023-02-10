# milli-py

Python bindings for Milli, the embeddable Rust-based search engine powering Meilisearch.

Due to limitations around Rust lifecycles, all functionality available in `IndexDocuments` and `Search` has been integrated as methods of the `Index` class. This ~~sacrifices~~ simplifies functionality available in the origianl *milli* package.

Install the package via:

```sh
pip install milli
```

## Usage

Basic usage of the package:

```py
import milli

index = milli.Index("path/to/index")
index.add_documents([
    { "title": "Hello world", "content": "This is a sample" },
    { "title": "Hello moon", "content": "This is another sample" },
    { "title": "Hello sun", "content": "This is yet another sample" },
])
results = index.search("wrold")
result = index.get_document(results[0])
assert(result['title'] == "Hello world")
```
