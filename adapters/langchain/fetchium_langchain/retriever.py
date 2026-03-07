"""Fetchium LangChain Retriever — wraps the Fetchium CLI or REST API."""

from __future__ import annotations

import json
import subprocess
from typing import Dict, List, Optional

try:
    from langchain_core.callbacks import CallbackManagerForRetrieverRun
    from langchain_core.documents import Document
    from langchain_core.retrievers import BaseRetriever
except ImportError as e:
    raise ImportError(
        "langchain-core is required. Install it with: pip install langchain-core"
    ) from e


class FetchiumRetriever(BaseRetriever):
    """LangChain Retriever that uses Fetchium for token-efficient web search.

    Can operate in two modes:
    - **CLI mode** (default): calls the ``fetchium`` binary via subprocess.
    - **REST mode**: calls a running ``fetchium serve --mode rest`` endpoint.
    """

    token_budget: int = 3000
    tier: str = "detailed"
    validate: bool = True
    max_sources: int = 10
    fetchium_binary: str = "fetchium"
    timeout: int = 60
    rest_base_url: Optional[str] = None
    rest_api_key: Optional[str] = None

    def _get_relevant_documents(
        self,
        query: str,
        *,
        run_manager: Optional[CallbackManagerForRetrieverRun] = None,
    ) -> List[Document]:
        if self.rest_base_url:
            return self._rest_search(query)
        return self._cli_search(query)

    def _cli_search(self, query: str) -> List[Document]:
        cmd: List[str] = [
            self.fetchium_binary,
            "agent-search",
            query,
            "--budget",
            str(self.token_budget),
            "--tier",
            self.tier,
            "--format",
            "json",
            "--max-sources",
            str(self.max_sources),
        ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=self.timeout,
            )
        except FileNotFoundError:
            raise RuntimeError(
                f"Fetchium binary not found at '{self.fetchium_binary}'. "
                "Install it with: cargo install fetchium-cli"
            )
        except subprocess.TimeoutExpired:
            raise RuntimeError(
                f"Fetchium timed out after {self.timeout}s. Increase the timeout for complex queries."
            )

        if result.returncode != 0:
            raise RuntimeError(
                f"Fetchium exited with code {result.returncode}: {result.stderr.strip()}"
            )

        return self._parse_cli_response(result.stdout)

    def _rest_search(self, query: str) -> List[Document]:
        try:
            import requests  # type: ignore[import]
        except ImportError:
            raise ImportError(
                "The 'requests' library is required for REST mode. Install it with: pip install requests"
            )

        headers: Dict[str, str] = {"Content-Type": "application/json"}
        if self.rest_api_key:
            headers["Authorization"] = f"Bearer {self.rest_api_key}"

        payload = {
            "query": query,
            "token_budget": self.token_budget,
            "tier": self.tier,
            "max_sources": self.max_sources,
            "validate": self.validate,
        }

        resp = requests.post(
            f"{self.rest_base_url.rstrip('/')}/v1/search",
            json=payload,
            headers=headers,
            timeout=self.timeout,
        )
        resp.raise_for_status()
        return self._parse_rest_response(resp.text)

    @staticmethod
    def _parse_cli_response(raw: str) -> List[Document]:
        try:
            data = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"Fetchium returned invalid JSON: {exc}") from exc

        documents: List[Document] = []
        for segment in data.get("segments", []):
            documents.append(
                Document(
                    page_content=segment.get("content", ""),
                    metadata={
                        "source": segment.get("source_url", ""),
                        "relevance": segment.get("relevance", 0),
                        "type": segment.get("type", "paragraph"),
                        "tokens": segment.get("tokens", 0),
                    },
                )
            )

        if not documents:
            for finding in data.get("findings", []):
                documents.append(
                    Document(
                        page_content=finding.get("claim", ""),
                        metadata={
                            "source": finding.get("source_url", ""),
                            "confidence": finding.get("confidence", 0.0),
                        },
                    )
                )

        return documents

    @staticmethod
    def _parse_rest_response(raw: str) -> List[Document]:
        try:
            data = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"Fetchium returned invalid JSON: {exc}") from exc

        return [
            Document(
                page_content=result.get("snippet") or result.get("title", ""),
                metadata={
                    "source": result.get("url", ""),
                    "title": result.get("title", ""),
                    "score": result.get("score", 0.0),
                    "query": data.get("meta", {}).get("query", ""),
                    "tier": data.get("meta", {}).get("tier", ""),
                    "result_id": data.get("meta", {}).get("result_id", ""),
                },
            )
            for result in data.get("results", [])
        ]
