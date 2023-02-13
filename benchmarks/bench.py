#!/usr/bin/env python3

import random
import string
import tempfile
import timeit

# Configuration
BENCH_TIMES = 1
DOC_COUNT = 100000
DOC_SIZE = 100

def gen_documents():
    documents = []
    characters = '     ' + string.ascii_lowercase
    random.seed(0)
    for i in range(DOC_COUNT):
        data = ''.join(random.choices(characters, k=DOC_SIZE))
        documents.append({ "id": 0, "data": data })
    return documents

def bench_milli(docs, target_query):
    import milli
    with tempfile.TemporaryDirectory() as tmp:
        ix = milli.Index(tmp)
        ix.add_documents(docs)
        t = timeit.timeit(lambda: ix.search(target_query), number=BENCH_TIMES) / BENCH_TIMES
        del(ix)
    return t

def bench_whoosh(docs, target_query):
    from whoosh.util.testing import TempIndex
    from whoosh.fields import Schema, TEXT
    from whoosh.index import create_in
    from whoosh.qparser import QueryParser
    schema = Schema(data=TEXT(stored=True))
    with TempIndex(schema) as ix:
        writer = ix.writer()
        for doc in docs:
            writer.add_document(data=doc['data'])
        writer.commit()
        with ix.searcher() as searcher:
            query = QueryParser("data", ix.schema).parse(target_query)
            t = timeit.timeit(lambda: searcher.search(query), number=BENCH_TIMES) / BENCH_TIMES
    return t

def main():
    print("Generating documents...")
    docs = gen_documents()
    target_docid = random.randint(0, DOC_COUNT)
    target_start = random.randint(1 * DOC_SIZE // 10,
                                  9 * DOC_SIZE // 10)
    target_query = docs[target_docid]['data'][target_start:target_start+(DOC_SIZE // 10)]

    print("Bencharking Milli...")
    t_milli = bench_milli(docs, target_query)
    print(f'Time per query: {t_milli}')

if __name__ == '__main__':
    main()
