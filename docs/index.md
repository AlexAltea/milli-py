# Documentation

## `Index`

Main class used to interface with an index/database in the local filesystem. An *index* is a directory containing a pair of `data.mdb` and `lock.mdb` files.

### Constructor

> *Index(path, max_size)*

Opens or creates an index at the specified directory, limiting the maximum size of the underlying databse.

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `path` | Yes | [Path-like object](https://docs.python.org/3/glossary.html#term-path-like-object) | Index directory. Directory must exist. |
| `max_size` | Yes | [`int`](https://docs.python.org/3/library/functions.html#int) | Maximum size in bytes of `data.mdb`. |

Example:

```py
>>> index = Index('path/to/index', 2**30)  # Open/create index of up-to 1 GiB in size
```

### Methods

#### `Index.add_documents`

> *Index.add_documents(documents)*

Adds documents to the index.

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `documents` | Yes | [`List[Dict[str,Any]]`](https://docs.python.org/3/library/typing.html#typing.List) | List of JSON-convertible dictionaries, i.e. dictionaries with string keys mapping to integers, floats, booleans, strings, arrays, and other dictionaries with string keys (potentially nested). |

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

Obtain a document from the index given its internal ID.

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `id` | Yes | [`int`](https://docs.python.org/3/library/functions.html#int) | Internal document ID. |

Returns: [`Dict[str,Any]`](https://docs.python.org/3/library/typing.html#typing.Dict). Document contents.

Example:

```py
>>> index.get_document(0)
{ 'id': 0, 'title': 'Hello earth', 'tags': ['greeting', 'planet'], 'orbit': 3 }
```

#### `Index.get_documents`

> *Index.get_documents(ids)*

Obtain a list of document from the index given their internal IDs.

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `ids` | Yes | [`List[int]`](https://docs.python.org/3/library/typing.html#typing.List) | List of internal document IDs. |

Returns: [`List[Dict[str,Any]]`](https://docs.python.org/3/library/typing.html#typing.List). List of document contents.

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

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `query` | Yes | [`str`](https://docs.python.org/3/library/stdtypes.html#str) | Text to query the index with. |

Returns: [`List[int]`](https://docs.python.org/3/library/typing.html#typing.List). List of internal IDs of matching documents, sorted by decreasing match score. You can retrieve the full documents by applying [`Index.get_documents`](#indexget_documents) on this list.

Example:

```py
>>> index.search('earht')
[0]
```
