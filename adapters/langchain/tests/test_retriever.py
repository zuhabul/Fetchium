import importlib
import json
import sys
import types
import unittest


def install_langchain_stubs():
    langchain_core = types.ModuleType("langchain_core")
    callbacks = types.ModuleType("langchain_core.callbacks")
    documents = types.ModuleType("langchain_core.documents")
    retrievers = types.ModuleType("langchain_core.retrievers")

    class CallbackManagerForRetrieverRun:
        pass

    class Document:
        def __init__(self, page_content="", metadata=None):
            self.page_content = page_content
            self.metadata = metadata or {}

    class BaseRetriever:
        pass

    callbacks.CallbackManagerForRetrieverRun = CallbackManagerForRetrieverRun
    documents.Document = Document
    retrievers.BaseRetriever = BaseRetriever

    sys.modules["langchain_core"] = langchain_core
    sys.modules["langchain_core.callbacks"] = callbacks
    sys.modules["langchain_core.documents"] = documents
    sys.modules["langchain_core.retrievers"] = retrievers


class FetchiumRetrieverTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        install_langchain_stubs()
        sys.path.insert(0, "adapters/langchain")
        cls.module = importlib.import_module("fetchium_langchain.retriever")

    def test_parse_rest_response(self):
        raw = json.dumps(
            {
                "meta": {"query": "rust", "tier": "summary", "result_id": "abc"},
                "results": [
                    {
                        "title": "Rust Book",
                        "url": "https://doc.rust-lang.org/book/",
                        "snippet": "The Rust Programming Language",
                        "score": 0.9,
                    }
                ],
            }
        )
        docs = self.module.FetchiumRetriever._parse_rest_response(raw)
        self.assertEqual(len(docs), 1)
        self.assertEqual(docs[0].metadata["source"], "https://doc.rust-lang.org/book/")
        self.assertEqual(docs[0].metadata["result_id"], "abc")

    def test_parse_cli_response(self):
        raw = json.dumps(
            {
                "segments": [
                    {
                        "content": "Ownership and borrowing",
                        "source_url": "https://example.com",
                        "relevance": 0.8,
                        "type": "paragraph",
                        "tokens": 42,
                    }
                ]
            }
        )
        docs = self.module.FetchiumRetriever._parse_cli_response(raw)
        self.assertEqual(len(docs), 1)
        self.assertEqual(docs[0].page_content, "Ownership and borrowing")


if __name__ == "__main__":
    unittest.main()
