#!/usr/bin/env python3
"""
Fetchium competitor benchmark harness.

Usage:
  export FETCHIUM_API_KEY="..."
  export SERPER_API_KEY="..."
  export EXA_API_KEY="..."
  export TAVILY_API_KEY="..."
  export FIRECRAWL_API_KEY="..."

  python fetchium_benchmark.py --out report.json
"""

import os
import json
import time
import uuid
import argparse
from dataclasses import dataclass, asdict
from typing import Any, Dict, List, Optional

import requests

TIMEOUT = 45

@dataclass
class CallResult:
    provider: str
    category: str
    name: str
    ok: bool
    status_code: Optional[int]
    latency_ms: Optional[float]
    error: Optional[str]
    response_preview: Any
    notes: Optional[str] = None

def timed_post(
    provider: str, category: str, name: str, url: str, headers: Dict[str, str], payload: Dict[str, Any], timeout: int = TIMEOUT
) -> CallResult:
    start = time.perf_counter()
    try:
        r = requests.post(url, headers=headers, json=payload, timeout=timeout)
        latency = (time.perf_counter() - start) * 1000
        preview = None
        try:
            data = r.json()
            if isinstance(data, dict):
                preview = {k: data.get(k) for k in list(data.keys())[:8]}
            else:
                preview = data[:3] if isinstance(data, list) else str(data)[:500]
        except Exception:
            preview = r.text[:500]
        return CallResult(
            provider=provider, category=category, name=name, ok=r.ok, status_code=r.status_code,
            latency_ms=round(latency, 2), error=None if r.ok else (r.text[:500] or f"HTTP {r.status_code}"),
            response_preview=preview
        )
    except Exception as e:
        latency = (time.perf_counter() - start) * 1000
        return CallResult(
            provider=provider, category=category, name=name, ok=False, status_code=None,
            latency_ms=round(latency, 2), error=str(e), response_preview=None
        )

def timed_get(
    provider: str, category: str, name: str, url: str, headers: Dict[str, str], timeout: int = TIMEOUT
) -> CallResult:
    start = time.perf_counter()
    try:
        r = requests.get(url, headers=headers, timeout=timeout)
        latency = (time.perf_counter() - start) * 1000
        preview = None
        try:
            data = r.json()
            if isinstance(data, dict):
                preview = {k: data.get(k) for k in list(data.keys())[:8]}
            else:
                preview = data[:3] if isinstance(data, list) else str(data)[:500]
        except Exception:
            preview = r.text[:500]
        return CallResult(
            provider=provider, category=category, name=name, ok=r.ok, status_code=r.status_code,
            latency_ms=round(latency, 2), error=None if r.ok else (r.text[:500] or f"HTTP {r.status_code}"),
            response_preview=preview
        )
    except Exception as e:
        latency = (time.perf_counter() - start) * 1000
        return CallResult(
            provider=provider, category=category, name=name, ok=False, status_code=None,
            latency_ms=round(latency, 2), error=str(e), response_preview=None
        )

def fetchium_suite(base_url: str, api_key: str) -> List[CallResult]:
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    out = []
    out.append(timed_get("fetchium", "utility", "usage", f"{base_url}/v1/usage", headers))
    out.append(timed_post("fetchium", "search", "search_basic", f"{base_url}/v1/search", headers, {"query": "What are the latest advances in retrieval augmented generation in 2026?", "max_results": 5}))
    out.append(timed_post("fetchium", "extract", "fetch_basic", f"{base_url}/v1/fetch", headers, {"url": "https://example.com", "format": "markdown", "token_budget": 300}))
    out.append(timed_post("fetchium", "extract", "scrape_basic", f"{base_url}/v1/scrape", headers, {"url": "https://example.com", "extract_level": "content"}))
    out.append(timed_post("fetchium", "utility", "estimate", f"{base_url}/v1/estimate", headers, {"url": "https://example.com"}))
    out.append(timed_post("fetchium", "research", "research_sync", f"{base_url}/v1/research", headers, {"query": "Summarize the current state of browser-use AI agents with citations.", "max_results": 6, "fetch_full_content": True}))
    out.append(timed_post("fetchium", "research", "research_job_submit", f"{base_url}/v1/research/jobs", headers, {"query": "Summarize the current state of browser-use AI agents with citations.", "max_results": 6, "fetch_full_content": True}))
    out.append(timed_post("fetchium", "youtube", "youtube_search", f"{base_url}/v1/youtube/search", headers, {"query": "rust programming tutorial", "max_results": 3}))
    out.append(timed_post("fetchium", "youtube", "youtube_analyze", f"{base_url}/v1/youtube/analyze", headers, {"url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ", "transcript": False, "comments": False, "teaching": False}))
    out.append(timed_post("fetchium", "social", "reddit_search", f"{base_url}/v1/social/reddit", headers, {"query": "Rust programming", "max_posts": 3}))
    out.append(timed_post("fetchium", "social", "hackernews_search", f"{base_url}/v1/social/hackernews", headers, {"query": "Rust", "max_results": 3}))
    out.append(timed_post("fetchium", "social", "social_research", f"{base_url}/v1/social/research", headers, {"query": "Rust programming", "platforms": ["reddit", "hackernews"], "max_per_platform": 2, "generate_ideas": False}))
    return out

def serper_suite(api_key: str) -> List[CallResult]:
    base_url = "https://google.serper.dev"
    headers = {"X-API-KEY": api_key, "Content-Type": "application/json"}
    out = []
    out.append(timed_post("serper", "search", "web_search", f"{base_url}/search", headers, {"q": "What are the latest advances in retrieval augmented generation in 2026?", "gl": "us", "hl": "en", "num": 5}))
    out.append(timed_post("serper", "search", "news_search", f"{base_url}/news", headers, {"q": "AI agent news", "gl": "us", "hl": "en", "num": 5}))
    out.append(timed_post("serper", "search", "video_search", f"{base_url}/videos", headers, {"q": "rust programming tutorial", "gl": "us", "hl": "en"}))
    return out

def exa_suite(api_key: str) -> List[CallResult]:
    base_url = "https://api.exa.ai"
    headers = {"x-api-key": api_key, "Content-Type": "application/json"}
    out = []
    out.append(timed_post("exa", "search", "search_auto", f"{base_url}/search", headers, {"query": "What are the latest advances in retrieval augmented generation in 2026?", "numResults": 5, "text": True}))
    out.append(timed_post("exa", "search", "search_deep", f"{base_url}/search", headers, {"query": "What are the latest advances in retrieval augmented generation in 2026?", "type": "deep", "numResults": 5, "contents": {"text": True}}))
    out.append(timed_post("exa", "research", "answer", f"{base_url}/answer", headers, {"query": "Summarize the state of browser-use AI agents.", "text": True}))
    return out

def tavily_suite(api_key: str) -> List[CallResult]:
    base_url = "https://api.tavily.com"
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    out = []
    out.append(timed_get("tavily", "utility", "usage", f"{base_url}/usage", headers))
    out.append(timed_post("tavily", "search", "search", f"{base_url}/search", headers, {"query": "What are the latest advances in retrieval augmented generation in 2026?", "search_depth": "advanced", "max_results": 5, "include_answer": True, "include_raw_content": True}))
    out.append(timed_post("tavily", "extract", "extract", f"{base_url}/extract", headers, {"urls": ["https://example.com"], "extract_depth": "basic"}))
    out.append(timed_post("tavily", "research", "research", f"{base_url}/research", headers, {"input": "Summarize the state of browser-use AI agents with citations.", "model": "mini", "stream": False}))
    return out

def firecrawl_suite(api_key: str) -> List[CallResult]:
    base_url = "https://api.firecrawl.dev/v2"
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    out = []
    out.append(timed_post("firecrawl", "search", "search", f"{base_url}/search", headers, {"query": "What are the latest advances in retrieval augmented generation in 2026?", "limit": 5, "sources": ["web"], "scrapeOptions": {"formats": ["markdown"]}}))
    out.append(timed_post("firecrawl", "extract", "scrape", f"{base_url}/scrape", headers, {"url": "https://example.com", "formats": ["markdown", "html", "rawHtml", "links"]}))
    return out

def summarize(results: List[CallResult]) -> Dict[str, Any]:
    by_provider = {}
    for r in results:
        p = by_provider.setdefault(r.provider, {"calls": 0, "ok_calls": 0, "latencies_ms": [], "categories": {}})
        p["calls"] += 1
        if r.ok: p["ok_calls"] += 1
        if r.latency_ms is not None: p["latencies_ms"].append(r.latency_ms)
        cat = p["categories"].setdefault(r.category, {"calls": 0, "ok_calls": 0})
        cat["calls"] += 1
        if r.ok: cat["ok_calls"] += 1
    for p, data in by_provider.items():
        lats = sorted(data["latencies_ms"])
        data["success_rate"] = round(data["ok_calls"] / data["calls"], 3) if data["calls"] else None
        data["avg_latency_ms"] = round(sum(lats) / len(lats), 2) if lats else None
        data["p50_latency_ms"] = lats[len(lats)//2] if lats else None
        data["p95_latency_ms"] = lats[min(len(lats)-1, max(0, int(len(lats)*0.95)-1))] if lats else None
    return by_provider

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default=f"benchmark_report_{uuid.uuid4().hex[:8]}.json")
    parser.add_argument("--fetchium-base", default="http://127.0.0.1:8080")
    args = parser.parse_args()

    results = []

    fk = os.getenv("FETCHIUM_API_KEY")
    sk = os.getenv("SERPER_API_KEY")
    ek = os.getenv("EXA_API_KEY")
    tk = os.getenv("TAVILY_API_KEY")
    ck = os.getenv("FIRECRAWL_API_KEY")

    if fk: results.extend(fetchium_suite(args.fetchium_base, fk))
    if sk: results.extend(serper_suite(sk))
    if ek: results.extend(exa_suite(ek))
    if tk: results.extend(tavily_suite(tk))
    if ck: results.extend(firecrawl_suite(ck))

    payload = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "summary": summarize(results),
        "results": [asdict(r) for r in results],
    }

    out_path = os.path.abspath(args.out)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, ensure_ascii=False)

    print(json.dumps(payload["summary"], indent=2))
    print(f"\nSaved detailed report to: {out_path}")

if __name__ == "__main__":
    main()
