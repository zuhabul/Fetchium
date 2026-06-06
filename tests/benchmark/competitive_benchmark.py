#!/usr/bin/env python3
"""Competitive benchmark: Fetchium vs Serper vs Exa vs Firecrawl vs Tavily.

Tests identical queries across all 5 services and compares:
- Result relevance (0-10 per query)
- Latency (ms)
- Result count
- Source diversity
- Content quality
"""

import json
import time
import requests
import sys
import os
from concurrent.futures import ThreadPoolExecutor, as_completed

# ── API Keys ────────────────────────────────────────────────
FETCHIUM_KEY = "fetchium_78ae2181bbcbd572b8bd092df26ae3b89cee6c01e053e6745b6261ae5bfbade5"
SERPER_KEY = "e88feb18b71987dde4301947f050604b22bb9363"
EXA_KEY = "eb4d14ea-602c-4a03-bf41-1c496f8bfb00"
FIRECRAWL_KEY = "fc-c3ed9e586f2e438f96653e98d3ddcac4"
TAVILY_KEY = "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"

FETCHIUM_URL = "http://localhost:3050/v1/search"

# ── Test Queries (30 across 10 categories) ──────────────────
BENCHMARK_QUERIES = {
    "factual": [
        "What is photosynthesis",
        "How does TCP/IP work",
        "What causes earthquakes",
    ],
    "how_to": [
        "how to get coffee stains out of white shirt",
        "how to set up a home Wi-Fi network",
        "how to start investing in stocks",
    ],
    "current_events": [
        "latest developments in AI regulation 2026",
        "SpaceX Starship launch updates",
        "climate change policy 2026",
    ],
    "comparison": [
        "rust vs go for backend development",
        "React vs Vue vs Svelte comparison",
        "PostgreSQL vs MySQL performance",
    ],
    "casual_everyday": [
        "best pizza in new york",
        "cheap flights to paris",
        "signs of burnout",
    ],
    "technical": [
        "kubernetes pod networking deep dive",
        "zero-knowledge proofs explained",
        "WebAssembly performance benchmarks",
    ],
    "opinion": [
        "best programming language for beginners 2026",
        "is AI going to replace software engineers",
        "best noise cancelling headphones",
    ],
    "academic": [
        "transformer architecture attention mechanism paper",
        "CRISPR gene editing recent breakthroughs",
        "quantum error correction techniques",
    ],
    "code": [
        "python asyncio tutorial with examples",
        "rust lifetime annotations explained",
        "implement binary search tree javascript",
    ],
    "multilingual": [
        "comment apprendre le francais rapidement",
        "como cocinar paella valenciana",
        "was ist quantencomputing einfach erklaert",
    ],
}

# ── Service Clients ─────────────────────────────────────────

def query_fetchium(query, max_results=5):
    """Query Fetchium API."""
    start = time.time()
    try:
        resp = requests.post(
            FETCHIUM_URL,
            headers={
                "Authorization": f"Bearer {FETCHIUM_KEY}",
                "Content-Type": "application/json",
            },
            json={"query": query, "max_sources": max_results},
            timeout=30,
        )
        latency = int((time.time() - start) * 1000)
        if resp.status_code != 200:
            return {"error": f"HTTP {resp.status_code}", "latency_ms": latency, "results": []}
        data = resp.json()
        results = []
        for r in data.get("results", []):
            results.append({
                "title": r.get("title", ""),
                "url": r.get("url", ""),
                "snippet": r.get("snippet", "")[:200],
            })
        return {"latency_ms": latency, "results": results, "error": None}
    except Exception as e:
        return {"error": str(e), "latency_ms": int((time.time() - start) * 1000), "results": []}


def query_serper(query, max_results=5):
    """Query Serper (Google SERP API)."""
    start = time.time()
    try:
        resp = requests.post(
            "https://google.serper.dev/search",
            headers={"X-API-KEY": SERPER_KEY, "Content-Type": "application/json"},
            json={"q": query, "num": max_results},
            timeout=15,
        )
        latency = int((time.time() - start) * 1000)
        if resp.status_code != 200:
            return {"error": f"HTTP {resp.status_code}", "latency_ms": latency, "results": []}
        data = resp.json()
        results = []
        for r in data.get("organic", [])[:max_results]:
            results.append({
                "title": r.get("title", ""),
                "url": r.get("link", ""),
                "snippet": r.get("snippet", "")[:200],
            })
        return {"latency_ms": latency, "results": results, "error": None}
    except Exception as e:
        return {"error": str(e), "latency_ms": int((time.time() - start) * 1000), "results": []}


def query_exa(query, max_results=5):
    """Query Exa (neural search)."""
    start = time.time()
    try:
        resp = requests.post(
            "https://api.exa.ai/search",
            headers={"x-api-key": EXA_KEY, "Content-Type": "application/json"},
            json={
                "query": query,
                "numResults": max_results,
                "type": "auto",
                "useAutoprompt": True,
                "contents": {"text": {"maxCharacters": 200}},
            },
            timeout=15,
        )
        latency = int((time.time() - start) * 1000)
        if resp.status_code != 200:
            return {"error": f"HTTP {resp.status_code}: {resp.text[:100]}", "latency_ms": latency, "results": []}
        data = resp.json()
        results = []
        for r in data.get("results", [])[:max_results]:
            results.append({
                "title": r.get("title", ""),
                "url": r.get("url", ""),
                "snippet": (r.get("text", "") or "")[:200],
            })
        return {"latency_ms": latency, "results": results, "error": None}
    except Exception as e:
        return {"error": str(e), "latency_ms": int((time.time() - start) * 1000), "results": []}


def query_tavily(query, max_results=5):
    """Query Tavily (AI search)."""
    start = time.time()
    try:
        resp = requests.post(
            "https://api.tavily.com/search",
            headers={"Content-Type": "application/json"},
            json={
                "api_key": TAVILY_KEY,
                "query": query,
                "max_results": max_results,
                "search_depth": "basic",
                "include_answer": False,
            },
            timeout=15,
        )
        latency = int((time.time() - start) * 1000)
        if resp.status_code != 200:
            return {"error": f"HTTP {resp.status_code}: {resp.text[:100]}", "latency_ms": latency, "results": []}
        data = resp.json()
        results = []
        for r in data.get("results", [])[:max_results]:
            results.append({
                "title": r.get("title", ""),
                "url": r.get("url", ""),
                "snippet": r.get("content", "")[:200],
            })
        return {"latency_ms": latency, "results": results, "error": None}
    except Exception as e:
        return {"error": str(e), "latency_ms": int((time.time() - start) * 1000), "results": []}


def query_firecrawl(query, max_results=5):
    """Query Firecrawl (web scraping + search, uses /search endpoint)."""
    start = time.time()
    try:
        # Firecrawl's search endpoint
        resp = requests.post(
            "https://api.firecrawl.dev/v1/search",
            headers={
                "Authorization": f"Bearer {FIRECRAWL_KEY}",
                "Content-Type": "application/json",
            },
            json={"query": query, "limit": max_results},
            timeout=20,
        )
        latency = int((time.time() - start) * 1000)
        if resp.status_code != 200:
            return {"error": f"HTTP {resp.status_code}: {resp.text[:100]}", "latency_ms": latency, "results": []}
        data = resp.json()
        results = []
        for r in data.get("data", [])[:max_results]:
            results.append({
                "title": r.get("title", r.get("metadata", {}).get("title", "")),
                "url": r.get("url", ""),
                "snippet": (r.get("markdown", "") or r.get("description", ""))[:200],
            })
        return {"latency_ms": latency, "results": results, "error": None}
    except Exception as e:
        return {"error": str(e), "latency_ms": int((time.time() - start) * 1000), "results": []}


# ── Relevance Scoring ───────────────────────────────────────

def score_relevance(query, results):
    """Score relevance of results for a query (0-10).

    Checks:
    - Do titles/snippets contain query terms? (term overlap)
    - Are results from diverse domains?
    - Are results non-empty?
    """
    if not results:
        return 0

    query_words = set(w.lower() for w in query.split() if len(w) > 2)
    query_words -= {"the", "and", "for", "how", "what", "why", "who", "are", "does",
                    "with", "from", "this", "that", "was", "has", "been", "its", "can"}

    if not query_words:
        query_words = set(w.lower() for w in query.split() if len(w) > 1)

    total_score = 0
    domains = set()

    for r in results:
        text = f"{r.get('title', '')} {r.get('snippet', '')}".lower()
        url = r.get("url", "")

        # Term overlap (0-2 points per result)
        matches = sum(1 for w in query_words if w in text)
        term_score = min(2.0, (matches / max(len(query_words), 1)) * 2)
        total_score += term_score

        # Extract domain for diversity
        try:
            from urllib.parse import urlparse
            domain = urlparse(url).netloc.replace("www.", "")
            domains.add(domain)
        except:
            pass

    # Normalize to 0-10 scale
    max_possible = len(results) * 2  # max 2 per result
    relevance = (total_score / max(max_possible, 1)) * 8  # 0-8 from relevance

    # Diversity bonus (0-2)
    diversity = min(2.0, len(domains) / max(len(results), 1) * 2)

    return round(min(10, relevance + diversity), 1)


# ── Main Benchmark ──────────────────────────────────────────

def run_benchmark():
    services = {
        "Fetchium": query_fetchium,
        "Serper": query_serper,
        "Exa": query_exa,
        "Tavily": query_tavily,
        "Firecrawl": query_firecrawl,
    }

    all_results = {}
    category_scores = {svc: {} for svc in services}
    category_latencies = {svc: {} for svc in services}

    total_queries = sum(len(qs) for qs in BENCHMARK_QUERIES.values())
    done = 0

    for category, queries in BENCHMARK_QUERIES.items():
        cat_scores = {svc: [] for svc in services}
        cat_latencies = {svc: [] for svc in services}

        for query in queries:
            done += 1
            print(f"\n[{done}/{total_queries}] {category}: {query}")

            query_results = {}

            # Run each service sequentially to avoid rate limits
            for svc_name, svc_fn in services.items():
                result = svc_fn(query)
                relevance = score_relevance(query, result["results"])
                query_results[svc_name] = {
                    "latency_ms": result["latency_ms"],
                    "result_count": len(result["results"]),
                    "relevance": relevance,
                    "error": result.get("error"),
                    "results": result["results"][:3],  # Save top 3 for analysis
                }
                cat_scores[svc_name].append(relevance)
                cat_latencies[svc_name].append(result["latency_ms"])

                status = "ERR" if result.get("error") else f"{relevance}/10"
                count = len(result["results"])
                ms = result["latency_ms"]
                print(f"  {svc_name:<12s} {status:>7s} | {count} results | {ms:>5d}ms")

                # Small delay to avoid rate limits
                time.sleep(0.3)

            all_results[f"{category}/{query}"] = query_results

        # Category averages
        for svc in services:
            scores = [s for s in cat_scores[svc] if s > 0]
            lats = [l for l in cat_latencies[svc] if l > 0]
            category_scores[svc][category] = round(sum(scores) / max(len(scores), 1), 1)
            category_latencies[svc][category] = int(sum(lats) / max(len(lats), 1))

    return all_results, category_scores, category_latencies


def print_report(all_results, category_scores, category_latencies):
    services = ["Fetchium", "Serper", "Exa", "Tavily", "Firecrawl"]
    categories = list(BENCHMARK_QUERIES.keys())

    print("\n" + "=" * 100)
    print("COMPETITIVE BENCHMARK RESULTS")
    print("=" * 100)

    # ── Category Relevance Table ──
    print("\n### RELEVANCE SCORES BY CATEGORY (0-10)")
    header = f"{'Category':<20s}" + "".join(f"{s:>12s}" for s in services)
    print(header)
    print("-" * len(header))

    totals = {s: [] for s in services}
    for cat in categories:
        row = f"{cat:<20s}"
        for svc in services:
            score = category_scores[svc].get(cat, 0)
            totals[svc].append(score)
            # Highlight winner
            best = max(category_scores[s].get(cat, 0) for s in services)
            marker = " *" if score == best and score > 0 else "  "
            row += f"{score:>10.1f}{marker}"
        print(row)

    print("-" * len(header))
    row = f"{'AVERAGE':<20s}"
    for svc in services:
        avg = sum(totals[svc]) / max(len(totals[svc]), 1)
        row += f"{avg:>10.1f}  "
    print(row)

    # ── Category Latency Table ──
    print("\n### LATENCY BY CATEGORY (ms)")
    header = f"{'Category':<20s}" + "".join(f"{s:>12s}" for s in services)
    print(header)
    print("-" * len(header))

    lat_totals = {s: [] for s in services}
    for cat in categories:
        row = f"{cat:<20s}"
        for svc in services:
            lat = category_latencies[svc].get(cat, 0)
            lat_totals[svc].append(lat)
            best = min(category_latencies[s].get(cat, 99999) for s in services if category_latencies[s].get(cat, 0) > 0)
            marker = " *" if lat == best and lat > 0 else "  "
            row += f"{lat:>10d}{marker}"
        print(row)

    print("-" * len(header))
    row = f"{'AVERAGE':<20s}"
    for svc in services:
        lats = [l for l in lat_totals[svc] if l > 0]
        avg = int(sum(lats) / max(len(lats), 1))
        row += f"{avg:>10d}  "
    print(row)

    # ── Per-query failure analysis ──
    print("\n### QUERY FAILURES / ERRORS")
    for key, qr in all_results.items():
        for svc in services:
            if qr[svc].get("error"):
                print(f"  [{svc}] {key}: {qr[svc]['error'][:80]}")

    # ── Head-to-head wins ──
    print("\n### HEAD-TO-HEAD WINS (which service had best relevance per query)")
    wins = {s: 0 for s in services}
    ties = 0
    for key, qr in all_results.items():
        scores = {s: qr[s]["relevance"] for s in services}
        best = max(scores.values())
        winners = [s for s, v in scores.items() if v == best and v > 0]
        if len(winners) == 1:
            wins[winners[0]] += 1
        else:
            ties += 1

    for svc in services:
        bar = "#" * wins[svc]
        print(f"  {svc:<12s} {wins[svc]:>3d} wins  {bar}")
    print(f"  {'Ties':<12s} {ties:>3d}")

    # ── Summary ──
    print("\n### OVERALL RANKING")
    overall = []
    for svc in services:
        avg_rel = sum(totals[svc]) / max(len(totals[svc]), 1)
        lats = [l for l in lat_totals[svc] if l > 0]
        avg_lat = int(sum(lats) / max(len(lats), 1))
        overall.append((svc, avg_rel, avg_lat, wins[svc]))

    # Sort by relevance (primary), then wins, then latency
    overall.sort(key=lambda x: (-x[1], -x[3], x[2]))

    for rank, (svc, rel, lat, w) in enumerate(overall, 1):
        print(f"  #{rank} {svc:<12s}  Relevance: {rel:.1f}/10  Latency: {lat}ms  Wins: {w}")


if __name__ == "__main__":
    print("=" * 100)
    print("FETCHIUM COMPETITIVE BENCHMARK")
    print(f"Testing 5 services × {sum(len(v) for v in BENCHMARK_QUERIES.values())} queries × 10 categories")
    print("=" * 100)

    results, scores, latencies = run_benchmark()
    print_report(results, scores, latencies)

    # Save raw data
    output_path = "/home/echo/projects/Fetchium/tests/benchmark/results.json"
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "w") as f:
        json.dump({
            "results": results,
            "category_scores": scores,
            "category_latencies": latencies,
        }, f, indent=2)
    print(f"\nRaw data saved to: {output_path}")
