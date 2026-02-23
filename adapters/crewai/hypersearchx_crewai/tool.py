"""HyperSearchX CrewAI Tool — wraps the hsx CLI or REST API."""

from __future__ import annotations

import json
import subprocess
from typing import Any, Dict, Optional, Type

try:
    from crewai.tools import BaseTool
    from pydantic import BaseModel, Field
except ImportError as e:
    raise ImportError(
        "crewai is required. Install it with: pip install crewai"
    ) from e


# ---------------------------------------------------------------------------
# Input schema for structured tool calls
# ---------------------------------------------------------------------------


class _SearchInput(BaseModel):
    """Input schema for HyperSearchXTool."""

    query: str = Field(..., description="The search query to execute.")


class _ResearchInput(BaseModel):
    """Input schema for HyperSearchXResearchTool."""

    query: str = Field(..., description="The research question or topic to investigate.")


# ---------------------------------------------------------------------------
# HyperSearchXTool — fast web search
# ---------------------------------------------------------------------------


class HyperSearchXTool(BaseTool):
    """CrewAI Tool that uses HyperSearchX for token-efficient web search.

    Returns a plain-text summary of findings suitable for CrewAI agent reasoning.

    Example::

        from hypersearchx_crewai import HyperSearchXTool

        tool = HyperSearchXTool(token_budget=2000, tier="summary")
        result = tool.run("latest Rust async framework benchmarks")
    """

    name: str = "HyperSearchX Web Search"
    description: str = (
        "Search the web using HyperSearchX. Returns token-efficient, "
        "validated results with citations. Input should be a search query string."
    )
    args_schema: Type[BaseModel] = _SearchInput

    token_budget: int = 2000
    """Maximum tokens to extract per search."""

    tier: str = "summary"
    """PDS tier: key_facts | summary | detailed | complete."""

    hsx_binary: str = "hsx"
    """Path to the hsx binary."""

    timeout: int = 60
    """Subprocess timeout in seconds."""

    rest_base_url: Optional[str] = None
    """If set, use REST API at this URL instead of the CLI subprocess."""

    rest_api_key: Optional[str] = None
    """Optional Bearer token for REST API authentication."""

    def _run(self, query: str) -> str:
        """Execute a search and return string output for the agent."""
        if self.rest_base_url:
            return self._rest_search(query)
        return self._cli_search(query)

    # ------------------------------------------------------------------
    # CLI mode
    # ------------------------------------------------------------------

    def _cli_search(self, query: str) -> str:
        cmd = [
            self.hsx_binary,
            "agent-search",
            query,
            "--budget",
            str(self.token_budget),
            "--tier",
            self.tier,
            "--format",
            "json",
        ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=self.timeout,
            )
        except FileNotFoundError:
            return (
                f"Error: HyperSearchX binary not found at '{self.hsx_binary}'. "
                "Install it with: cargo install hsx-cli"
            )
        except subprocess.TimeoutExpired:
            return f"Error: HyperSearchX timed out after {self.timeout}s."

        if result.returncode != 0:
            return f"Search failed: {result.stderr.strip()}"

        return self._format_response(result.stdout)

    # ------------------------------------------------------------------
    # REST mode
    # ------------------------------------------------------------------

    def _rest_search(self, query: str) -> str:
        try:
            import requests  # type: ignore[import]
        except ImportError:
            return "Error: 'requests' library required for REST mode. Run: pip install requests"

        headers: Dict[str, str] = {"Content-Type": "application/json"}
        if self.rest_api_key:
            headers["Authorization"] = f"Bearer {self.rest_api_key}"

        payload = {
            "query": query,
            "budget": self.token_budget,
            "tier": self.tier,
        }

        try:
            resp = requests.post(
                f"{self.rest_base_url.rstrip('/')}/api/search",
                json=payload,
                headers=headers,
                timeout=self.timeout,
            )
            resp.raise_for_status()
            return self._format_response(resp.text)
        except Exception as exc:
            return f"REST API error: {exc}"

    # ------------------------------------------------------------------
    # Response formatting
    # ------------------------------------------------------------------

    @staticmethod
    def _format_response(raw: str) -> str:
        """Format hsx JSON as a plain-text string for the CrewAI agent."""
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return raw  # Return raw if not valid JSON

        output_parts = []

        # Try findings first
        for finding in data.get("findings", []):
            claim = finding.get("claim", "")
            source = finding.get("source_url", "N/A")
            confidence = finding.get("confidence", "")
            if claim:
                conf_str = f" (confidence: {confidence:.0%})" if confidence else ""
                output_parts.append(f"- {claim}{conf_str} [Source: {source}]")

        # Fallback to segments if no findings
        if not output_parts:
            for seg in data.get("segments", []):
                content = seg.get("content", "").strip()
                source = seg.get("source_url", "N/A")
                if content:
                    output_parts.append(f"[{source}]\n{content}")

        return "\n".join(output_parts) if output_parts else "No results found."


# ---------------------------------------------------------------------------
# HyperSearchXResearchTool — deep multi-source research
# ---------------------------------------------------------------------------


class HyperSearchXResearchTool(BaseTool):
    """CrewAI Tool for deep multi-source research using HyperSearchX.

    Unlike ``HyperSearchXTool`` (fast web search), this tool runs the full
    AMRS research pipeline — query decomposition, parallel source fetching,
    cross-source validation, and citation-backed synthesis. Suitable for
    complex questions that require thorough investigation.

    Example::

        from hypersearchx_crewai import HyperSearchXResearchTool

        tool = HyperSearchXResearchTool(max_sources=20)
        result = tool.run("Compare Tokio vs async-std for production Rust services")
    """

    name: str = "HyperSearchX Deep Research"
    description: str = (
        "Conduct deep multi-source research using HyperSearchX. "
        "Decomposes the query, searches multiple sources, validates claims, "
        "and returns a synthesized report with citations. Use this for complex "
        "questions requiring thorough investigation."
    )
    args_schema: Type[BaseModel] = _ResearchInput

    max_sources: int = 20
    """Maximum sources to include in the research."""

    hsx_binary: str = "hsx"
    """Path to the hsx binary."""

    timeout: int = 120
    """Subprocess timeout in seconds (research takes longer than search)."""

    rest_base_url: Optional[str] = None
    """If set, use REST API at this URL instead of the CLI subprocess."""

    rest_api_key: Optional[str] = None
    """Optional Bearer token for REST API authentication."""

    def _run(self, query: str) -> str:
        """Execute deep research and return synthesized findings."""
        if self.rest_base_url:
            return self._rest_research(query)
        return self._cli_research(query)

    def _cli_research(self, query: str) -> str:
        cmd = [
            self.hsx_binary,
            "agent-research",
            query,
            "--max-sources",
            str(self.max_sources),
            "--format",
            "json",
        ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=self.timeout,
            )
        except FileNotFoundError:
            return (
                f"Error: HyperSearchX binary not found at '{self.hsx_binary}'. "
                "Install it with: cargo install hsx-cli"
            )
        except subprocess.TimeoutExpired:
            return f"Error: Research timed out after {self.timeout}s. Try reducing --max-sources."

        if result.returncode != 0:
            return f"Research failed: {result.stderr.strip()}"

        return self._format_research_response(result.stdout)

    def _rest_research(self, query: str) -> str:
        try:
            import requests  # type: ignore[import]
        except ImportError:
            return "Error: 'requests' library required for REST mode. Run: pip install requests"

        headers: Dict[str, str] = {"Content-Type": "application/json"}
        if self.rest_api_key:
            headers["Authorization"] = f"Bearer {self.rest_api_key}"

        payload = {"query": query, "max_sources": self.max_sources}

        try:
            resp = requests.post(
                f"{self.rest_base_url.rstrip('/')}/api/research",
                json=payload,
                headers=headers,
                timeout=self.timeout,
            )
            resp.raise_for_status()
            return self._format_research_response(resp.text)
        except Exception as exc:
            return f"REST API error: {exc}"

    @staticmethod
    def _format_research_response(raw: str) -> str:
        """Format the agent-research JSON response for CrewAI."""
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return raw

        parts = []

        # Summary
        summary = data.get("summary", "")
        if summary:
            parts.append(f"Summary:\n{summary}")

        # Key findings
        findings = data.get("findings", [])
        if findings:
            parts.append("\nKey Findings:")
            for i, finding in enumerate(findings, 1):
                claim = finding.get("claim", "")
                source = finding.get("source_url", "N/A")
                confidence = finding.get("confidence", 0.0)
                if claim:
                    parts.append(
                        f"{i}. {claim} (confidence: {confidence:.0%}) — {source}"
                    )

        # Contradictions
        contradictions = data.get("contradictions", [])
        if contradictions:
            parts.append("\nContradictions Detected:")
            for c in contradictions:
                parts.append(f"- {c}")

        return "\n".join(parts) if parts else "No research results found."
