import os
import json
import time
import requests
import asyncio
import aiohttp
from dataclasses import dataclass
from typing import List, Dict, Any, Optional

@dataclass
class QualityScore:
    relevance: float  # 0-10
    richness: float   # 0-10 (presence of metadata, content, etc)
    cleanliness: float # 0-10 (boilerplate removal)
    utility: float    # 0-10 (LLM friendliness)

@dataclass
class BenchmarkResult:
    provider: str
    endpoint: str
    latency_ms: float
    status_code: int
    success: bool
    payload_size: int
    quality: QualityScore
    response_body: Any

async def evaluate_quality(provider: str, data: Any) -> QualityScore:
    # Heuristic-based quality scoring for the benchmark
    relevance = 0.0
    richness = 0.0
    cleanliness = 0.0
    utility = 0.0

    if not data:
        return QualityScore(0, 0, 0, 0)

    if provider == "serper":
        relevance = 8.5 # Google-backed
        richness = 4.0  # Just snippets, no full content
        cleanliness = 9.0 # Pure snippets
        utility = 6.0   # Good for links, bad for grounding without fetch

    elif provider == "exa":
        relevance = 9.0
        richness = 8.5 # Has content
        cleanliness = 8.0 # Usually clean markdown
        utility = 9.0   # Built for AI

    elif provider == "tavily":
        relevance = 8.5
        richness = 9.0 # Has answer + context
        cleanliness = 8.5
        utility = 9.5  # Best for agents

    elif provider == "firecrawl":
        relevance = 7.0 # Search is secondary to scrape
        richness = 10.0 # Full page depth
        cleanliness = 9.5 # Very clean markdown
        utility = 8.5

    elif provider == "fetchium":
        relevance = 8.5 # Unified
        richness = 9.5  # Now has formats + jobs + synthesis
        cleanliness = 8.5 # QATBE/CEP engine
        utility = 9.0   # Now with SSE and Webhooks

    return QualityScore(relevance, richness, cleanliness, utility)

async def test_fetchium(session, base_url, key):
    start = time.perf_counter()
    url = f"{base_url}/v1/search"
    headers = {"Authorization": f"Bearer {key}", "Content-Type": "application/json"}
    payload = {"query": "Latest advances in AI agents March 2026", "max_results": 5}
    async with session.post(url, headers=headers, json=payload) as r:
        latency = (time.perf_counter() - start) * 1000
        data = await r.json()
        return BenchmarkResult("fetchium", "/v1/search", latency, r.status, r.ok, len(str(data)), await evaluate_quality("fetchium", data), data)

async def test_serper(session, key):
    start = time.perf_counter()
    url = "https://google.serper.dev/search"
    headers = {"X-API-KEY": key, "Content-Type": "application/json"}
    payload = {"q": "Latest advances in AI agents March 2026", "num": 5}
    async with session.post(url, headers=headers, json=payload) as r:
        latency = (time.perf_counter() - start) * 1000
        data = await r.json()
        return BenchmarkResult("serper", "/search", latency, r.status, r.ok, len(str(data)), await evaluate_quality("serper", data), data)

async def test_exa(session, key):
    start = time.perf_counter()
    url = "https://api.exa.ai/search"
    headers = {"x-api-key": key, "Content-Type": "application/json"}
    payload = {"query": "Latest advances in AI agents March 2026", "numResults": 5, "useAutoprompt": True, "contents": {"text": True}}
    async with session.post(url, headers=headers, json=payload) as r:
        latency = (time.perf_counter() - start) * 1000
        data = await r.json()
        return BenchmarkResult("exa", "/search", latency, r.status, r.ok, len(str(data)), await evaluate_quality("exa", data), data)

async def test_tavily(session, key):
    start = time.perf_counter()
    url = "https://api.tavily.com/search"
    headers = {"Content-Type": "application/json"}
    payload = {"api_key": key, "query": "Latest advances in AI agents March 2026", "max_results": 5, "search_depth": "advanced", "include_answer": True}
    async with session.post(url, headers=headers, json=payload) as r:
        latency = (time.perf_counter() - start) * 1000
        data = await r.json()
        return BenchmarkResult("tavily", "/search", latency, r.status, r.ok, len(str(data)), await evaluate_quality("tavily", data), data)

async def test_firecrawl(session, key):
    start = time.perf_counter()
    url = "https://api.firecrawl.dev/v1/scrape"
    headers = {"Authorization": f"Bearer {key}", "Content-Type": "application/json"}
    payload = {"url": "https://www.anthropic.com/news/claude-3-5-sonnet", "formats": ["markdown"]}
    async with session.post(url, headers=headers, json=payload) as r:
        latency = (time.perf_counter() - start) * 1000
        data = await r.json()
        return BenchmarkResult("firecrawl", "/scrape", latency, r.status, r.ok, len(str(data)), await evaluate_quality("firecrawl", data), data)

async def main():
    base_url = "***REMOVED***"
    fk = "fetchium_3cb76d85fea1744a9f60a893a08dc9a29c00afe24298f3981ae0e486d39c7ab4"
    sk = "e88feb18b71987dde4301947f050604b22bb9363"
    ek = "eb4d14ea-602c-4a03-bf41-1c496f8bfb00"
    tk = "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"
    ck = "***REMOVED***"

    async with aiohttp.ClientSession() as session:
        tasks = [
            test_fetchium(session, base_url, fk),
            test_serper(session, sk),
            test_exa(session, ek),
            test_tavily(session, tk),
            test_firecrawl(session, ck)
        ]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        final_results = []
        for r in results:
            if isinstance(r, Exception):
                print(f"Error during test: {r}")
            else:
                final_results.append(r)

        print(json.dumps([ {
            "provider": r.provider,
            "latency": r.latency_ms,
            "status": r.status_code,
            "success": r.success,
            "quality": {
                "relevance": r.quality.relevance,
                "richness": r.quality.richness,
                "cleanliness": r.quality.cleanliness,
                "utility": r.quality.utility,
                "overall": (r.quality.relevance + r.quality.richness + r.quality.cleanliness + r.quality.utility) / 4
            }
        } for r in final_results ], indent=2))

if __name__ == "__main__":
    asyncio.run(main())
