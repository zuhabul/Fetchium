#!/usr/bin/env python3
"""
Deep competitive benchmark: Fetchium vs Serper vs Exa vs Tavily vs Firecrawl.

Scoring: Gemini Flash evaluates each result set for true relevance (0-10).
Dimensions: relevance, source diversity, snippet richness, latency, coverage.
"""

import json, time, requests, sys, os, re
from concurrent.futures import ThreadPoolExecutor, as_completed

# ── API Keys ─────────────────────────────────────────────────────────────────
FETCHIUM_KEY = "***REMOVED***"
SERPER_KEY   = "***REMOVED***"
EXA_KEY      = "***REMOVED***"
TAVILY_KEY   = "***REMOVED***"
FIRECRAWL_KEY= "***REMOVED***"
GEMINI_KEY   = "***REMOVED***"

FETCHIUM_URL = "http://localhost:3050"

# ── Test Queries: 40 across 10 categories ────────────────────────────────────
QUERIES = {
    "factual": [
        "What is photosynthesis and how does it work",
        "How does TCP/IP protocol work",
        "What causes earthquakes and how are they measured",
        "How does mRNA vaccine technology work",
    ],
    "how_to": [
        "how to get coffee stains out of white shirt",
        "how to set up a home Wi-Fi network step by step",
        "how to start investing in index funds",
        "how to improve sleep quality naturally",
    ],
    "current_events": [
        "latest AI regulation news 2026",
        "SpaceX Starship mission updates 2026",
        "US economy inflation 2026",
        "climate change policy updates 2026",
    ],
    "comparison": [
        "rust vs go for backend development 2025",
        "PostgreSQL vs MySQL vs SQLite performance comparison",
        "iPhone vs Android which is better 2025",
        "React vs Vue vs Svelte frontend framework comparison",
    ],
    "casual_everyday": [
        "best pizza restaurants in new york city",
        "cheap flights to tokyo from london",
        "signs you might have burnout",
        "best noise cancelling headphones under 200",
    ],
    "technical": [
        "kubernetes pod networking deep dive",
        "zero-knowledge proofs cryptography explained",
        "WebAssembly WASM performance vs JavaScript benchmarks",
        "LLM transformer attention mechanism implementation",
    ],
    "opinion": [
        "best programming language for beginners in 2025",
        "is remote work better than office work",
        "which electric car brand is most reliable 2025",
        "best investment strategies for long term wealth",
    ],
    "academic": [
        "transformer architecture self-attention mechanism research",
        "CRISPR gene editing recent breakthroughs 2024 2025",
        "quantum error correction techniques overview",
        "retrieval augmented generation RAG for LLMs paper",
    ],
    "code": [
        "python asyncio concurrent programming tutorial",
        "rust lifetime annotations and borrow checker explained",
        "implement LRU cache in python",
        "javascript promise async await patterns",
    ],
    "multilingual": [
        "comment apprendre le francais rapidement",
        "como cocinar paella valenciana autentica",
        "was ist quantencomputing einfach erklaert",
        "mejores restaurantes en madrid 2025",
    ],
}

# ── Service Query Functions ───────────────────────────────────────────────────

def query_fetchium(query):
    t0 = time.time()
    last_err = None
    for attempt in range(3):
        try:
            if attempt > 0:
                time.sleep(2 * attempt)
            r = requests.post(f"{FETCHIUM_URL}/v1/search",
                json={"query": query, "max_results": 10},
                headers={"Authorization": f"Bearer {FETCHIUM_KEY}",
                         "Content-Type": "application/json"},
                timeout=30)
            data = r.json()
            results = []
            for item in data.get("results", []):
                results.append({
                    "title": item.get("title", ""),
                    "url": item.get("url", ""),
                    "snippet": item.get("snippet", "")[:400],
                })
            return {"results": results, "latency_ms": int((time.time()-t0)*1000), "raw": data}
        except Exception as e:
            last_err = e
    return {"results": [], "latency_ms": int((time.time()-t0)*1000), "error": str(last_err)}


def query_serper(query):
    t0 = time.time()
    try:
        r = requests.post("https://google.serper.dev/search",
            json={"q": query, "num": 10},
            headers={"X-API-KEY": SERPER_KEY, "Content-Type": "application/json"},
            timeout=15)
        data = r.json()
        results = []
        for item in data.get("organic", []):
            results.append({
                "title": item.get("title", ""),
                "url": item.get("link", ""),
                "snippet": item.get("snippet", ""),
            })
        return {"results": results, "latency_ms": int((time.time()-t0)*1000)}
    except Exception as e:
        return {"results": [], "latency_ms": int((time.time()-t0)*1000), "error": str(e)}


def query_exa(query):
    t0 = time.time()
    try:
        r = requests.post("https://api.exa.ai/search",
            json={"query": query, "numResults": 10, "type": "auto",
                  "useAutoprompt": True, "contents": {"text": {"maxCharacters": 300}}},
            headers={"x-api-key": EXA_KEY, "Content-Type": "application/json"},
            timeout=20)
        data = r.json()
        results = []
        for item in data.get("results", []):
            results.append({
                "title": item.get("title", ""),
                "url": item.get("url", ""),
                "snippet": (item.get("text") or item.get("snippet") or item.get("highlights", [""])[0] if item.get("highlights") else "")[:300],
            })
        return {"results": results, "latency_ms": int((time.time()-t0)*1000)}
    except Exception as e:
        return {"results": [], "latency_ms": int((time.time()-t0)*1000), "error": str(e)}


def query_tavily(query):
    t0 = time.time()
    try:
        r = requests.post("https://api.tavily.com/search",
            json={"query": query, "max_results": 10, "search_depth": "advanced",
                  "include_raw_content": False, "include_answer": False},
            headers={"Authorization": f"Bearer {TAVILY_KEY}",
                     "Content-Type": "application/json"},
            timeout=20)
        data = r.json()
        results = []
        for item in data.get("results", []):
            results.append({
                "title": item.get("title", ""),
                "url": item.get("url", ""),
                "snippet": item.get("content", "")[:300],
            })
        return {"results": results, "latency_ms": int((time.time()-t0)*1000)}
    except Exception as e:
        return {"results": [], "latency_ms": int((time.time()-t0)*1000), "error": str(e)}


def query_firecrawl(query):
    t0 = time.time()
    try:
        r = requests.post("https://api.firecrawl.dev/v1/search",
            json={"query": query, "limit": 10, "scrapeOptions": {"formats": []}},
            headers={"Authorization": f"Bearer {FIRECRAWL_KEY}",
                     "Content-Type": "application/json"},
            timeout=25)
        data = r.json()
        results = []
        for item in data.get("data", []):
            results.append({
                "title": item.get("title", ""),
                "url": item.get("url", ""),
                "snippet": item.get("description", "")[:300],
            })
        return {"results": results, "latency_ms": int((time.time()-t0)*1000)}
    except Exception as e:
        return {"results": [], "latency_ms": int((time.time()-t0)*1000), "error": str(e)}


# ── AI-Powered Scoring via Gemini Flash ──────────────────────────────────────

def gemini_score(query, results_by_service):
    """Ask Gemini to score each service's results for this query."""
    if not results_by_service:
        return {}

    services_text = ""
    for svc, results in results_by_service.items():
        if not results:
            services_text += f"\n## {svc}: NO RESULTS\n"
            continue
        services_text += f"\n## {svc} ({len(results)} results):\n"
        for i, r in enumerate(results[:7], 1):
            title = r.get("title", "")[:80]
            snippet = r.get("snippet", "")[:150]
            url = r.get("url", "")[:80]
            services_text += f"{i}. [{title}]({url})\n   {snippet}\n"

    prompt = f"""You are evaluating search engine results for quality and relevance.

Query: "{query}"

{services_text}

Score each service on these criteria (each 0-10):
1. Relevance: How well do results match what the user wants?
2. Source Quality: Are sources authoritative, reputable, and appropriate?
3. Snippet Richness: Do snippets give useful information vs being generic?
4. Coverage: Do results cover different angles/perspectives on the topic?

IMPORTANT: Respond ONLY with valid JSON, no markdown, no explanation:
{{
  "scores": {{
    "ServiceName": {{"relevance": 8.5, "source_quality": 7.0, "snippet_richness": 6.5, "coverage": 8.0, "total": 7.5}},
    ...
  }},
  "winner": "ServiceName",
  "fetchium_gaps": "specific things Fetchium is missing or worse at compared to winner (be specific)",
  "fetchium_strengths": "what Fetchium does better than others"
}}"""

    try:
        import subprocess
        result = subprocess.run(
            ["gemini", "-p", prompt],
            capture_output=True, text=True, timeout=60
        )
        text = result.stdout.strip()
        # Strip markdown code blocks if present
        text = re.sub(r'^```json\s*', '', text)
        text = re.sub(r'^```\s*', '', text)
        text = re.sub(r'\s*```$', '', text.strip())
        # Extract JSON object if surrounded by other text
        match = re.search(r'\{[\s\S]*\}', text)
        if match:
            text = match.group(0)
        return json.loads(text)
    except Exception as e:
        return {"error": str(e), "scores": {}}


# ── Helper metrics ────────────────────────────────────────────────────────────

def analyze_results(results):
    """Compute source diversity, snippet length, trusted domain hits."""
    if not results:
        return {"domain_count": 0, "avg_snippet_len": 0, "trusted_domains": 0}

    from urllib.parse import urlparse
    TRUSTED = {"wikipedia.org","stackoverflow.com","github.com","medium.com",
               "reddit.com","arxiv.org","nature.com","sciencedirect.com",
               "nytimes.com","bbc.com","reuters.com","techcrunch.com",
               "developer.mozilla.org","docs.python.org","rust-lang.org",
               "healthline.com","mayoclinic.org","investopedia.com",
               "youtube.com","coursera.org","udemy.com","w3schools.com"}

    domains = set()
    trusted = 0
    snippet_lens = []
    for r in results:
        try:
            d = urlparse(r.get("url","")).netloc.replace("www.","")
            domains.add(d)
            if any(t in d for t in TRUSTED):
                trusted += 1
        except: pass
        snippet_lens.append(len(r.get("snippet","") or ""))

    return {
        "domain_count": len(domains),
        "avg_snippet_len": int(sum(snippet_lens)/max(len(snippet_lens),1)),
        "trusted_domains": trusted,
    }


# ── Main Benchmark ────────────────────────────────────────────────────────────

def run():
    services = {
        "Fetchium":  query_fetchium,
        "Serper":    query_serper,
        "Exa":       query_exa,
        "Tavily":    query_tavily,
        "Firecrawl": query_firecrawl,
    }
    svc_names = list(services.keys())

    all_data = {}
    cat_ai_scores = {s: {} for s in svc_names}
    cat_latencies = {s: {} for s in svc_names}
    fetchium_gaps = []
    fetchium_strengths = []

    total = sum(len(qs) for qs in QUERIES.values())
    done = 0

    for category, queries in QUERIES.items():
        print(f"\n{'='*80}")
        print(f"  CATEGORY: {category.upper()}")
        print(f"{'='*80}")
        cat_scores_acc = {s: [] for s in svc_names}
        cat_lat_acc = {s: [] for s in svc_names}

        for query in queries:
            done += 1
            print(f"\n[{done}/{total}] {query}")
            print("-" * 60)

            svc_results = {}
            svc_latencies = {}

            # Fetch from all services in parallel
            with ThreadPoolExecutor(max_workers=5) as ex:
                futures = {ex.submit(fn, query): name for name, fn in services.items()}
                for fut in as_completed(futures):
                    name = futures[fut]
                    res = fut.result()
                    svc_results[name] = res.get("results", [])
                    svc_latencies[name] = res.get("latency_ms", 0)
                    err = res.get("error", "")
                    count = len(svc_results[name])
                    lat = svc_latencies[name]
                    print(f"  {name:<12s} {count:>2d} results | {lat:>5d}ms{'  ERR: '+err[:50] if err else ''}")

            # AI scoring
            print("  → Gemini scoring...", end="", flush=True)
            ai = gemini_score(query, {s: svc_results[s] for s in svc_names})
            print(" done")

            ai_scores = ai.get("scores", {})
            winner = ai.get("winner", "?")
            gap = ai.get("fetchium_gaps", "")
            strength = ai.get("fetchium_strengths", "")

            if gap:
                fetchium_gaps.append(f"[{category}] {query[:50]}: {gap}")
            if strength:
                fetchium_strengths.append(f"[{category}] {strength}")

            print(f"  Winner: {winner}")
            for s in svc_names:
                sc = ai_scores.get(s, {})
                total_sc = sc.get("total", 0)
                rel = sc.get("relevance", 0)
                qual = sc.get("source_quality", 0)
                snip = sc.get("snippet_richness", 0)
                cov = sc.get("coverage", 0)
                mark = " ★" if s == winner else ""
                print(f"    {s:<12s} total={total_sc:.1f}  rel={rel:.1f}  qual={qual:.1f}  snip={snip:.1f}  cov={cov:.1f}{mark}")

            # Accumulate
            for s in svc_names:
                sc = ai_scores.get(s, {})
                t = sc.get("total", 0)
                if t > 0:
                    cat_scores_acc[s].append(t)
                    cat_lat_acc[s].append(svc_latencies.get(s, 0))

            # Analysis metrics
            for s in svc_names:
                m = analyze_results(svc_results[s])
                all_data[f"{category}/{query}"] = all_data.get(f"{category}/{query}", {})
                all_data[f"{category}/{query}"][s] = {
                    "latency_ms": svc_latencies.get(s, 0),
                    "result_count": len(svc_results[s]),
                    "ai_score": ai_scores.get(s, {}),
                    **m,
                    "top3": svc_results[s][:3],
                }

            time.sleep(0.5)  # rate limit buffer

        # Category summary
        for s in svc_names:
            sc = cat_scores_acc[s]
            lt = cat_lat_acc[s]
            cat_ai_scores[s][category] = round(sum(sc)/max(len(sc),1), 1)
            cat_latencies[s][category] = int(sum(lt)/max(len(lt),1))

    return all_data, cat_ai_scores, cat_latencies, fetchium_gaps, fetchium_strengths


def print_report(all_data, cat_scores, cat_lats, gaps, strengths):
    svc_names = ["Fetchium", "Serper", "Exa", "Tavily", "Firecrawl"]
    categories = list(QUERIES.keys())

    print("\n\n" + "="*100)
    print("DEEP COMPETITIVE BENCHMARK — FINAL REPORT")
    print("="*100)

    # Relevance table
    print("\n### AI-SCORED RELEVANCE BY CATEGORY (0-10, scored by Gemini Flash)")
    hdr = f"{'Category':<22}" + "".join(f"{s:>12}" for s in svc_names)
    print(hdr)
    print("-" * len(hdr))
    totals = {s: [] for s in svc_names}
    for cat in categories:
        best = max(cat_scores[s].get(cat, 0) for s in svc_names)
        row = f"{cat:<22}"
        for s in svc_names:
            sc = cat_scores[s].get(cat, 0)
            totals[s].append(sc)
            mark = "★" if sc == best and sc > 0 else " "
            row += f"{sc:>10.1f}{mark} "
        print(row)
    print("-" * len(hdr))
    row = f"{'AVERAGE':<22}"
    for s in svc_names:
        avg = sum(totals[s]) / max(len(totals[s]), 1)
        row += f"{avg:>10.1f}  "
    print(row)

    # Latency table
    print("\n### LATENCY BY CATEGORY (ms) — lower is better")
    hdr = f"{'Category':<22}" + "".join(f"{s:>12}" for s in svc_names)
    print(hdr)
    print("-" * len(hdr))
    lat_totals = {s: [] for s in svc_names}
    for cat in categories:
        valid_lats = {s: cat_lats[s].get(cat, 0) for s in svc_names if cat_lats[s].get(cat, 0) > 0}
        best_lat = min(valid_lats.values()) if valid_lats else 0
        row = f"{cat:<22}"
        for s in svc_names:
            lt = cat_lats[s].get(cat, 0)
            lat_totals[s].append(lt)
            mark = "★" if lt == best_lat and lt > 0 else " "
            row += f"{lt:>10}{mark} "
        print(row)
    print("-" * len(hdr))
    row = f"{'AVERAGE':<22}"
    for s in svc_names:
        lts = [l for l in lat_totals[s] if l > 0]
        avg = int(sum(lts)/max(len(lts),1))
        row += f"{avg:>10}  "
    print(row)

    # Head-to-head wins
    print("\n### HEAD-TO-HEAD WINS (per query, by AI score)")
    wins = {s: 0 for s in svc_names}
    ties = 0
    for key, qd in all_data.items():
        scores = {}
        for s in svc_names:
            sc = qd.get(s, {}).get("ai_score", {})
            scores[s] = sc.get("total", 0)
        best = max(scores.values())
        if best == 0:
            continue
        winners = [s for s, v in scores.items() if v == best]
        if len(winners) == 1:
            wins[winners[0]] += 1
        else:
            ties += 1
    for s in svc_names:
        bar = "█" * wins[s]
        print(f"  {s:<12} {wins[s]:>3} wins  {bar}")
    print(f"  {'Ties':<12} {ties:>3}")

    # Overall ranking
    print("\n### OVERALL RANKING")
    ranking = []
    for s in svc_names:
        avg_rel = sum(totals[s]) / max(len(totals[s]), 1)
        lts = [l for l in lat_totals[s] if l > 0]
        avg_lat = int(sum(lts)/max(len(lts),1))
        ranking.append((s, avg_rel, avg_lat, wins[s]))
    ranking.sort(key=lambda x: (-x[1], -x[3], x[2]))
    for rank, (s, rel, lat, w) in enumerate(ranking, 1):
        crown = " 👑" if rank == 1 else ""
        print(f"  #{rank} {s:<12}  Score: {rel:.2f}/10  Latency: {lat}ms  Wins: {w}{crown}")

    # Fetchium gap analysis
    print("\n### FETCHIUM GAP ANALYSIS (where we lose to competitors)")
    if gaps:
        seen = set()
        for g in gaps:
            key = g[:60]
            if key not in seen:
                seen.add(key)
                print(f"  ⚠ {g[:120]}")
    else:
        print("  None found — Fetchium is dominant!")

    print("\n### FETCHIUM STRENGTHS")
    if strengths:
        seen = set()
        for s in strengths[:10]:
            key = s[:60]
            if key not in seen:
                seen.add(key)
                print(f"  ✓ {s[:120]}")

    print("\n" + "="*100)


if __name__ == "__main__":
    print("="*100)
    print("FETCHIUM DEEP COMPETITIVE BENCHMARK")
    print(f"Services: Fetchium vs Serper vs Exa vs Tavily vs Firecrawl")
    print(f"Queries: {sum(len(v) for v in QUERIES.values())} across {len(QUERIES)} categories")
    print(f"Scoring: Gemini Flash AI evaluation (0-10 per dimension)")
    print("="*100)

    data, scores, lats, gaps, strengths = run()
    print_report(data, scores, lats, gaps, strengths)

    out = "/home/echo/projects/Fetchium/tests/benchmark/deep_results.json"
    with open(out, "w") as f:
        json.dump({"data": data, "scores": scores, "latencies": lats,
                   "gaps": gaps, "strengths": strengths}, f, indent=2)
    print(f"\nFull results: {out}")
