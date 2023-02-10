# milli-py

Python bindings for [Milli](https://github.com/meilisearch/meilisearch/tree/main/milli), the embeddable Rust-based search engine powering [Meilisearch](https://www.meilisearch.com/).

Due to limitations around Rust lifecycles, methods available via `IndexDocuments` and `Search` have been integrated directly into the `Index` class. This ~~sacrifices~~ simplifies functionality in the original *milli* package.

Install the package via:

```sh
pip install milli
```

## Usage

Basic usage of the *milli-py*:

```py
import milli

index = milli.Index("path/to/index")
index.add_documents([
    { "title": "Hello world", "content": "This is a sample" },
    { "title": "Hello moon", "content": "This is another sample" },
    { "title": "Hello sun", "content": "This is yet another sample" },
])
results = index.search("wrold")
document = index.get_document(results[0])
assert(document['title'] == "Hello world")
```
