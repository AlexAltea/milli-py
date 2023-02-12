# milli-py

[![ci-badge](https://github.com/AlexAltea/milli-py/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexAltea/milli-py/actions/workflows/ci.yml)
[![](https://img.shields.io/pypi/v/milli.svg)](https://pypi.python.org/pypi/milli)

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
    { "id": 0, "title": "Hello world", "content": "This is a sample" },
    { "id": 1, "title": "Hello moon", "content": "This is another sample" },
    { "id": 2, "title": "Hello sun", "content": "This is yet another sample" },
])
results = index.search("wrold")
document = index.get_document(results[0])
assert(document['title'] == "Hello world")
```

## Examples

Real-world examples of *milli-py* in action:

- [Curator](https://github.com/AlexAltea/curator/blob/master/curator/databases/omdb.py#L9): Searching 140k movies (~10 MB csv, ~140 MB index) in around 36ms to obtain metadata from OMDB and auto-rename/tag movie collections.

Do you have other interesting examples? Let me know!
