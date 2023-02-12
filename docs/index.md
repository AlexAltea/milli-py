# Documentation

TODO.

## `Index`

Main class used to interface with an index/database in the local filesystem. An *index* is a directory containing a pair of `data.mdb` and `lock.mdb` files.

### Constructor

> *Index(path, max_size)*

Opens or creates an index at the specified directory, limiting the maximum size of the underlying databse.

Arguments:

- `path`: Path to the index directory provided as a string. Directory must exists.
- `max_size`: Maximum size in bytes of `data.mdb`.

Example:

```py
>>> index = Index('path/to/index', 2**30)  # Open/create index of up-to 1 GiB in size
```

### Methods

#### `Index.add_documents`

> *Index.add_documents(documents)*

Adds documents to the index.

Arguments:

- `documents`: List of JSON-convertible dictionaries, i.e. dictionaries with string keys mapping to integers, floats, booleans, strings, arrays, and other dictionaries with string keys (potentially nested).

Returns: TODO.

Example:

```py
>>> index.add_documents([
    { 'id': 0, 'title': 'Hello earth', 'tags': ['greeting', 'planet'], 'orbit': 3 },
    { 'id': 1, 'title': 'Hello mars', 'tags': ['greeting', 'planet'], 'orbit': 4 },
    { 'id': 2, 'title': 'Hello sun', 'tags': ['greeting', 'star'] },
])
```

#### `Index.get_document`

> *Index.get_document(id)*

Obtain the entire document from the index given its internal ID.

Arguments:

- `id`: Integer representing the document internal ID. 

Returns: Document contents

Example:

```py
>>> index.get_document(0)
{ 'id': 0, 'title': 'Hello earth', 'tags': ['greeting', 'planet'], 'orbit': 3 }
```

#### `Index.get_documents`

> *Index.get_documents(id)*

**Not yet implemented.**

Example (formatted):
```py
>>> index.get_documents([1,2])
[
    { 'id': 1, 'title': 'Hello mars', 'tags': ['greeting', 'planet'], 'orbit': 4 },
    { 'id': 2, 'title': 'Hello sun', 'tags': ['greeting', 'star'] }
]
```

#### `Index.search`

> *Index.search(query)*

Searches the index for the given input string.

Arguments:

- `query`: String to query the index with.

Returns: List of internal IDs of matching documents, sorted by decreasing match score. You can retrieve the full documents by applying [`Index.get_documents`](#indexget_document) on this list.

Example:

```py
>>> index.search('earht')
[0]
```
