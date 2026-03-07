"""Fetchium CrewAI tools — wrap the Fetchium CLI or REST API."""

from __future__ import annotations

import json
import subprocess
from typing import Dict, Optional, Type

try:
    from crewai.tools import BaseTool
    from pydantic import BaseModel, Field
except ImportError as e:
    raise ImportError("crewai is required. Install it with: pip install crewai") from e


class _SearchInput(BaseModel):
    """Input schema for FetchiumTool."""

    query: str = Field(..., description="The search query to execute.")


class _ResearchInput(BaseModel):
    """Input schema for FetchiumResearchTool."""

    query: str = Field(..., description="The research question or topic to investigate.")


class FetchiumTool(BaseTool):
    """CrewAI tool that uses Fetchium for token-efficient web search."""

    name: str = "Fetchium Web Search"
    description: str = (
        "Search the web using Fetchium. Returns token-efficient, validated results with citations."
    )
    args_schema: Type[BaseModel] = _SearchInput

    token_budget: int = 2000
    tier: str = "summary"
    fetchium_binary: str = "fetchium"
    timeout: int = 60
    rest_base_url: Optional[str] = None
    rest_api_key: Optional[str] = None

    def _run(self, query: str) -> str:
        if self.rest_base_url:
            return self._rest_search(query)
        return self._cli_search(query)

    def _cli_search(self, query: str) -> str:
        cmd = [
            self.fetchium_binary,
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
                f"Error: Fetchium binary not found at '{self.fetchium_binary}'. "
                "Install it with: cargo install fetchium-cli"
            )
        except subprocess.TimeoutExpired:
            return f"Error: Fetchium timed out after {self.timeout}s."

        if result.returncode != 0:
            return f"Search failed: {result.stderr.strip()}"

        return self._format_cli_response(result.stdout)

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
            "token_budget": self.token_budget,
            "tier": self.tier,
        }

        try:
            resp = requests.post(
                f"{self.rest_base_url.rstrip('/')}/v1/search",
                json=payload,
                headers=headers,
                timeout=self.timeout,
            )
            resp.raise_for_status()
            return self._format_rest_search_response(resp.text)
        except Exception as exc:
            return f"REST API error: {exc}"

    @staticmethod
    def _format_cli_response(raw: str) -> str:
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return raw

        output_parts = []
        for finding in data.get("findings", []):
            claim = finding.get("claim", "")
            source = finding.get("source_url", "N/A")
            confidence = finding.get("confidence", "")
            if claim:
                conf_str = f" (confidence: {confidence:.0%})" if confidence else ""
                output_parts.append(f"- {claim}{conf_str} [Source: {source}]")

        if not output_parts:
            for seg in data.get("segments", []):
                content = seg.get("content", "").strip()
                source = seg.get("source_url", "N/A")
                if content:
                    output_parts.append(f"[{source}]\n{content}")

        return "\n".join(output_parts) if output_parts else "No results found."

    @staticmethod
    def _format_rest_search_response(raw: str) -> str:
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return raw

        parts = []
        for result in data.get("results", []):
            title = result.get("title", "")
            snippet = result.get("snippet", "")
            url = result.get("url", "N/A")
            if title or snippet:
                parts.append(f"- {title}\n  {snippet}\n  Source: {url}")
        return "\n".join(parts) if parts else "No results found."


class FetchiumResearchTool(BaseTool):
    """CrewAI tool for multi-source research using Fetchium."""

    name: str = "Fetchium Deep Research"
    description: str = (
        "Conduct deep multi-source research using Fetchium and return a synthesized report with citations."
    )
    args_schema: Type[BaseModel] = _ResearchInput

    max_sources: int = 20
    fetchium_binary: str = "fetchium"
    timeout: int = 120
    rest_base_url: Optional[str] = None
    rest_api_key: Optional[str] = None

    def _run(self, query: str) -> str:
        if self.rest_base_url:
            return self._rest_research(query)
        return self._cli_research(query)

    def _cli_research(self, query: str) -> str:
        cmd = [
            self.fetchium_binary,
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
                f"Error: Fetchium binary not found at '{self.fetchium_binary}'. "
                "Install it with: cargo install fetchium-cli"
            )
        except subprocess.TimeoutExpired:
            return f"Error: Research timed out after {self.timeout}s. Try reducing max_sources."

        if result.returncode != 0:
            return f"Research failed: {result.stderr.strip()}"

        return self._format_cli_research_response(result.stdout)

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
                f"{self.rest_base_url.rstrip('/')}/v1/research",
                json=payload,
                headers=headers,
                timeout=self.timeout,
            )
            resp.raise_for_status()
            return self._format_rest_research_response(resp.text)
        except Exception as exc:
            return f"REST API error: {exc}"

    @staticmethod
    def _format_cli_research_response(raw: str) -> str:
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return raw

        parts = []
        findings = data.get("findings", [])
        if findings:
            parts.append("Key Findings:")
            for i, finding in enumerate(findings, 1):
                claim = finding.get("claim", "")
                confidence = finding.get("confidence", 0.0)
                if claim:
                    parts.append(f"{i}. {claim} (confidence: {confidence:.0%})")
        return "\n".join(parts) if parts else "No research results found."

    @staticmethod
    def _format_rest_research_response(raw: str) -> str:
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return raw

        parts = []
        report = data.get("report", "")
        if report:
            parts.append(report)
        refs = data.get("reference_section", "")
        if refs:
            parts.append(f"\nReferences:\n{refs}")
        return "\n".join(parts) if parts else "No research results found."
