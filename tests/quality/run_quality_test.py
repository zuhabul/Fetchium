#!/usr/bin/env python3
"""Comprehensive quality test suite for Fetchium search API.
Runs 100+ queries across 20 domains, scores relevance, reports failures."""

import json
import subprocess
import sys
import time
import re
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

API_URL = "http://127.0.0.1:4567/v1/search"
API_KEY = "fetchium_1f82fcf3425d0b9999daef4795e3104d032e20aba63a48923ef37d6f0bf22fed"
MAX_SOURCES = 5
TIMEOUT = 30  # seconds per query

def run_query(query_info, category):
    """Run a single search query and return results with quality metrics."""
    q = query_info["q"]
    expect_terms = [t.lower() for t in query_info.get("expect_terms", [])]

    payload = json.dumps({"query": q, "max_sources": MAX_SOURCES})
    try:
        result = subprocess.run(
            ["curl", "-s", "--max-time", str(TIMEOUT),
             "-H", f"Authorization: Bearer {API_KEY}",
             "-H", "Content-Type: application/json",
             "-d", payload, API_URL],
            capture_output=True, text=True, timeout=TIMEOUT + 5
        )
        data = json.loads(result.stdout)
    except (json.JSONDecodeError, subprocess.TimeoutExpired) as e:
        return {
            "query": q, "category": category, "status": "ERROR",
            "error": str(e), "results": [], "scores": {},
            "duration_ms": 0, "result_count": 0
        }

    meta = data.get("meta", {})
    results = data.get("results", [])
    duration_ms = meta.get("duration_ms", 0)

    # Quality scoring
    relevance_scores = []
    for r in results:
        title = (r.get("title") or "").lower()
        snippet = (r.get("snippet") or "").lower()
        url = (r.get("url") or "").lower()
        combined = f"{title} {snippet} {url}"

        # Count how many expected terms appear
        matched = sum(1 for t in expect_terms if t in combined)
        term_score = matched / max(len(expect_terms), 1)
        relevance_scores.append(term_score)

    avg_relevance = sum(relevance_scores) / max(len(relevance_scores), 1)
    top1_relevance = relevance_scores[0] if relevance_scores else 0
    top3_relevance = sum(relevance_scores[:3]) / min(3, max(len(relevance_scores), 1))

    # Score spread (good ranking should have spread)
    api_scores = [r.get("score", 0) for r in results]
    score_spread = (max(api_scores) - min(api_scores)) if len(api_scores) >= 2 else 0

    # Garbage detection: results completely unrelated to query
    garbage_count = sum(1 for s in relevance_scores if s == 0)

    # Duplicate URL detection
    urls = [r.get("url", "") for r in results]
    unique_urls = len(set(urls))
    has_dupes = unique_urls < len(urls)

    # Domain diversity
    domains = set()
    for u in urls:
        match = re.search(r'https?://(?:www\.)?([^/]+)', u)
        if match:
            domains.add(match.group(1))
    domain_diversity = len(domains) / max(len(urls), 1)

    # Status determination
    if not results:
        status = "EMPTY"
    elif avg_relevance >= 0.6 and garbage_count == 0:
        status = "EXCELLENT"
    elif avg_relevance >= 0.4 and garbage_count <= 1:
        status = "GOOD"
    elif avg_relevance >= 0.2:
        status = "FAIR"
    else:
        status = "POOR"

    return {
        "query": q,
        "category": category,
        "status": status,
        "duration_ms": duration_ms,
        "result_count": len(results),
        "scores": {
            "avg_relevance": round(avg_relevance, 3),
            "top1_relevance": round(top1_relevance, 3),
            "top3_relevance": round(top3_relevance, 3),
            "score_spread": round(score_spread, 3),
            "garbage_count": garbage_count,
            "has_dupes": has_dupes,
            "domain_diversity": round(domain_diversity, 3),
        },
        "results": [
            {
                "title": r.get("title", "")[:80],
                "url": r.get("url", ""),
                "score": r.get("score", 0),
                "term_match": round(relevance_scores[i], 3) if i < len(relevance_scores) else 0
            }
            for i, r in enumerate(results)
        ]
    }


def main():
    test_file = Path(__file__).parent / "test_queries.json"
    with open(test_file) as f:
        test_data = json.load(f)

    # Flatten all queries
    all_queries = []
    for category, queries in test_data["categories"].items():
        for q in queries:
            all_queries.append((q, category))

    print(f"\n{'='*80}")
    print(f"FETCHIUM QUALITY TEST SUITE")
    print(f"{'='*80}")
    print(f"Total queries: {len(all_queries)}")
    print(f"Categories: {len(test_data['categories'])}")
    print(f"API: {API_URL}")
    print(f"{'='*80}\n")

    # Run queries sequentially to avoid overwhelming the API
    results = []
    start_time = time.time()

    for idx, (q, cat) in enumerate(all_queries):
        result = run_query(q, cat)
        results.append(result)
        # Live progress
        status_icon = {
            "EXCELLENT": "\033[92m+\033[0m",
            "GOOD": "\033[93m~\033[0m",
            "FAIR": "\033[93m?\033[0m",
            "POOR": "\033[91m!\033[0m",
            "EMPTY": "\033[91mX\033[0m",
            "ERROR": "\033[91mE\033[0m",
        }.get(result["status"], "?")
        print(f"  [{status_icon}] {result['status']:9s} | {result['duration_ms']:5d}ms | "
              f"rel={result['scores'].get('avg_relevance', 0):.2f} | "
              f"{result['category']:20s} | {result['query'][:55]}", flush=True)
        # Small delay between queries
        time.sleep(0.3)

    total_time = time.time() - start_time

    # Analysis
    print(f"\n{'='*80}")
    print(f"RESULTS SUMMARY")
    print(f"{'='*80}")

    status_counts = {}
    for r in results:
        status_counts[r["status"]] = status_counts.get(r["status"], 0) + 1

    total = len(results)
    for status in ["EXCELLENT", "GOOD", "FAIR", "POOR", "EMPTY", "ERROR"]:
        count = status_counts.get(status, 0)
        pct = count / total * 100
        bar = "#" * int(pct / 2)
        print(f"  {status:10s}: {count:3d} ({pct:5.1f}%) {bar}")

    excellent_good = status_counts.get("EXCELLENT", 0) + status_counts.get("GOOD", 0)
    quality_rate = excellent_good / total * 100

    # Per-category breakdown
    print(f"\n{'='*80}")
    print(f"PER-CATEGORY BREAKDOWN")
    print(f"{'='*80}")
    print(f"{'Category':25s} | {'Total':5s} | {'Exc':4s} | {'Good':4s} | {'Fair':4s} | {'Poor':4s} | {'Empty':5s} | {'AvgRel':6s} | {'AvgMs':6s}")
    print("-" * 100)

    categories = {}
    for r in results:
        cat = r["category"]
        if cat not in categories:
            categories[cat] = {"results": [], "statuses": {}}
        categories[cat]["results"].append(r)
        s = r["status"]
        categories[cat]["statuses"][s] = categories[cat]["statuses"].get(s, 0) + 1

    for cat in sorted(categories.keys()):
        info = categories[cat]
        n = len(info["results"])
        avg_rel = sum(r["scores"].get("avg_relevance", 0) for r in info["results"]) / n
        avg_ms = sum(r["duration_ms"] for r in info["results"]) / n
        exc = info["statuses"].get("EXCELLENT", 0)
        good = info["statuses"].get("GOOD", 0)
        fair = info["statuses"].get("FAIR", 0)
        poor = info["statuses"].get("POOR", 0)
        empty = info["statuses"].get("EMPTY", 0)
        print(f"  {cat:23s} | {n:5d} | {exc:4d} | {good:4d} | {fair:4d} | {poor:4d} | {empty:5d} | {avg_rel:6.3f} | {avg_ms:6.0f}")

    # Failures detail
    failures = [r for r in results if r["status"] in ("POOR", "EMPTY", "ERROR")]
    if failures:
        print(f"\n{'='*80}")
        print(f"FAILURE DETAILS ({len(failures)} queries)")
        print(f"{'='*80}")
        for r in sorted(failures, key=lambda x: x["scores"].get("avg_relevance", 0)):
            print(f"\n  [{r['status']}] {r['category']}: \"{r['query']}\"")
            print(f"    Relevance: avg={r['scores'].get('avg_relevance', 0):.3f} "
                  f"top1={r['scores'].get('top1_relevance', 0):.3f} "
                  f"garbage={r['scores'].get('garbage_count', 0)}")
            if r.get("error"):
                print(f"    Error: {r['error']}")
            for i, res in enumerate(r.get("results", [])[:5]):
                print(f"    #{i+1} [match={res['term_match']:.2f}] {res['title'][:60]}")
                print(f"        {res['url'][:70]}")

    # Garbage results analysis
    garbage_queries = [r for r in results if r["scores"].get("garbage_count", 0) >= 2]
    if garbage_queries:
        print(f"\n{'='*80}")
        print(f"HIGH GARBAGE QUERIES ({len(garbage_queries)} queries with 2+ irrelevant results)")
        print(f"{'='*80}")
        for r in garbage_queries:
            print(f"  [{r['status']}] \"{r['query']}\" - {r['scores']['garbage_count']} garbage results")

    # Latency analysis
    durations = [r["duration_ms"] for r in results if r["duration_ms"] > 0]
    if durations:
        durations.sort()
        print(f"\n{'='*80}")
        print(f"LATENCY ANALYSIS")
        print(f"{'='*80}")
        print(f"  p50: {durations[len(durations)//2]}ms")
        print(f"  p90: {durations[int(len(durations)*0.9)]}ms")
        print(f"  p99: {durations[int(len(durations)*0.99)]}ms")
        print(f"  max: {max(durations)}ms")
        print(f"  min: {min(durations)}ms")
        print(f"  avg: {sum(durations)/len(durations):.0f}ms")
        slow = [r for r in results if r["duration_ms"] > 8000]
        if slow:
            print(f"  SLOW (>8s): {len(slow)} queries")
            for r in slow:
                print(f"    {r['duration_ms']}ms | {r['query'][:50]}")

    # Duplicate detection
    dupe_queries = [r for r in results if r["scores"].get("has_dupes")]
    if dupe_queries:
        print(f"\n  DUPLICATE URLS: {len(dupe_queries)} queries have duplicate result URLs")

    # Final score
    print(f"\n{'='*80}")
    print(f"FINAL SCORECARD")
    print(f"{'='*80}")
    print(f"  Quality Rate (Excellent+Good): {quality_rate:.1f}%")
    print(f"  Total Test Time: {total_time:.1f}s")
    print(f"  Average Latency: {sum(durations)/len(durations):.0f}ms" if durations else "")
    print(f"  Queries with Garbage: {len(garbage_queries)}/{total}")
    print(f"  Empty Results: {status_counts.get('EMPTY', 0)}/{total}")
    print(f"  Errors: {status_counts.get('ERROR', 0)}/{total}")

    target = 85.0
    if quality_rate >= target:
        print(f"\n  PASS - Quality rate {quality_rate:.1f}% >= {target}% target")
    else:
        print(f"\n  FAIL - Quality rate {quality_rate:.1f}% < {target}% target")
        print(f"  Need {int((target/100 * total) - excellent_good)} more queries at GOOD+ level")

    # Save full results
    output_file = Path(__file__).parent / "results" / f"quality_run_{int(time.time())}.json"
    output_file.parent.mkdir(exist_ok=True)
    with open(output_file, "w") as f:
        json.dump({
            "summary": {
                "total": total,
                "quality_rate": quality_rate,
                "status_counts": status_counts,
                "avg_latency_ms": sum(durations)/len(durations) if durations else 0,
                "total_time_s": total_time,
            },
            "results": results
        }, f, indent=2)
    print(f"\n  Full results saved to: {output_file}")
    print(f"{'='*80}\n")

    return 0 if quality_rate >= target else 1

if __name__ == "__main__":
    sys.exit(main())
