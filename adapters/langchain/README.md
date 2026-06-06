# fetchium-langchain

LangChain retriever adapter for Fetchium.

## Install

```bash
pip install fetchium-langchain
```

REST mode:

```bash
pip install fetchium-langchain[rest]
```

## Usage

```python
from fetchium_langchain import FetchiumRetriever

retriever = FetchiumRetriever(
    rest_base_url="https://api.fetchium.com",
    rest_api_key="fetchium_...",
    token_budget=2000,
    max_sources=5,
)

docs = retriever.invoke("Rust async runtimes")
```

CLI mode works if `fetchium` is installed on PATH.
