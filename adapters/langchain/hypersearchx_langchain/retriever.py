"""HyperSearchX LangChain Retriever — wraps the hsx CLI or REST API."""

from __future__ import annotations

import json
import subprocess
from typing import Any, Dict, List, Optional

try:
    from langchain_core.documents import Document
    from langchain_core.retrievers import BaseRetriever
    from langchain_core.callbacks import CallbackManagerForRetrieverRun
except ImportError as e:
    raise ImportError(
        "langchain-core is required. Install it with: pip install langchain-core"
    ) from e


class HyperSearchXRetriever(BaseRetriever):
    """LangChain Retriever that uses HyperSearchX for token-efficient web search.

    Can operate in two modes:
    - **CLI mode** (default): calls the ``hsx`` binary via subprocess.
    - **REST mode**: calls a running ``hsx serve --api`` endpoint over HTTP.

    Example::

        from hypersearchx_langchain import HyperSearchXRetriever

        retriever = HyperSearchXRetriever(token_budget=3000, max_sources=10)
        docs = retriever.invoke("Rust async runtime comparison 2025")
    """

    # --- Configuration fields (Pydantic v2 style, compatible with LangChain v0.2+) ---

    token_budget: int = 3000
    """Maximum tokens to extract per search."""

    tier: str = "detailed"
    """PDS tier: key_facts | summary | detailed | complete."""

    validate: bool = True
    """Whether to run cross-source validation."""

    max_sources: int = 10
    """Maximum number of sources to fetch."""

    hsx_binary: str = "hsx"
    """Path to the hsx binary (or just 'hsx' if it is on PATH)."""

    timeout: int = 60
    """Subprocess timeout in seconds."""

    rest_base_url: Optional[str] = None
    """If set, use REST API at this base URL instead of the CLI subprocess.
    Example: ``http://localhost:3000``"""

    rest_api_key: Optional[str] = None
    """Optional Bearer token for REST API authentication."""

    def _get_relevant_documents(
        self,
        query: str,
        *,
        run_manager: Optional[CallbackManagerForRetrieverRun] = None,
    ) -> List[Document]:
        """Retrieve documents for *query* using HyperSearchX."""
        if self.rest_base_url:
            return self._rest_search(query)
        return self._cli_search(query)

    # ------------------------------------------------------------------
    # CLI mode
    # ------------------------------------------------------------------

    def _cli_search(self, query: str) -> List[Document]:
        """Execute ``hsx agent-search`` as a subprocess and parse JSON output."""
        cmd: List[str] = [
            self.hsx_binary,
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
                f"HyperSearchX binary not found at '{self.hsx_binary}'. "
                "Install it with: cargo install hsx-cli  OR  set hsx_binary to the full path."
            )
        except subprocess.TimeoutExpired:
            raise RuntimeError(
                f"HyperSearchX timed out after {self.timeout}s. "
                "Increase the 'timeout' parameter for complex queries."
            )

        if result.returncode != 0:
            raise RuntimeError(
                f"HyperSearchX exited with code {result.returncode}: {result.stderr.strip()}"
            )

        return self._parse_response(result.stdout)

    # ------------------------------------------------------------------
    # REST mode
    # ------------------------------------------------------------------

    def _rest_search(self, query: str) -> List[Document]:
        """Call ``POST /api/search`` on a running hsx REST server."""
        try:
            import requests  # type: ignore[import]
        except ImportError:
            raise ImportError(
                "The 'requests' library is required for REST mode. "
                "Install it with: pip install requests"
            )

        headers: Dict[str, str] = {"Content-Type": "application/json"}
        if self.rest_api_key:
            headers["Authorization"] = f"Bearer {self.rest_api_key}"

        payload = {
            "query": query,
            "budget": self.token_budget,
            "tier": self.tier,
            "max_sources": self.max_sources,
        }

        resp = requests.post(
            f"{self.rest_base_url.rstrip('/')}/api/search",
            json=payload,
            headers=headers,
            timeout=self.timeout,
        )
        resp.raise_for_status()
        return self._parse_response(resp.text)

    # ------------------------------------------------------------------
    # Response parsing (shared between CLI and REST modes)
    # ------------------------------------------------------------------

    @staticmethod
    def _parse_response(raw: str) -> List[Document]:
        """Convert the hsx JSON response into LangChain ``Document`` objects."""
        try:
            data = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"HyperSearchX returned invalid JSON: {exc}") from exc

        documents: List[Document] = []

        # agent-search JSON shape: { "segments": [...], "findings": [...], ... }
        for segment in data.get("segments", []):
            doc = Document(
                page_content=segment.get("content", ""),
                metadata={
                    "source": segment.get("source_url", ""),
                    "relevance": segment.get("relevance", 0),
                    "type": segment.get("type", "paragraph"),
                    "tokens": segment.get("tokens", 0),
                },
            )
            documents.append(doc)

        # Fallback: if no segments, surface findings as plain documents
        if not documents:
            for finding in data.get("findings", []):
                doc = Document(
                    page_content=finding.get("claim", ""),
                    metadata={
                        "source": finding.get("source_url", ""),
                        "confidence": finding.get("confidence", 0.0),
                    },
                )
                documents.append(doc)

        return documents
