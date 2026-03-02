import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Python SDK" };

export default function PythonSDK() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">SDKs & Integrations</div>
      <h1>Python</h1>
      <p>
        Use Fetchium from Python with <code>requests</code>.
      </p>

      <h2>Client helper</h2>
      <CodeBlock language="python" filename="fetchium_client.py" code={`import os
import requests

FETCHIUM_BASE = "https://api.fetchium.com"
FETCHIUM_KEY = os.environ["FETCHIUM_API_KEY"]
HEADERS = {
    "Authorization": f"Bearer {FETCHIUM_KEY}",
    "Content-Type": "application/json",
}


def search(query: str) -> dict:
    r = requests.post(
        f"{FETCHIUM_BASE}/v1/search",
        headers=HEADERS,
        json={"query": query, "tier": "summary", "max_sources": 8},
        timeout=30,
    )
    r.raise_for_status()
    return r.json()


def scrape(url: str) -> dict:
    r = requests.post(
        f"{FETCHIUM_BASE}/v1/scrape",
        headers=HEADERS,
        json={"url": url, "format": "markdown", "token_budget": 3000},
        timeout=30,
    )
    r.raise_for_status()
    return r.json()


def research(query: str) -> dict:
    r = requests.post(
        f"{FETCHIUM_BASE}/v1/research",
        headers=HEADERS,
        json={"query": query, "depth": "standard", "citation_style": "inline"},
        timeout=120,
    )
    r.raise_for_status()
    return r.json()`} />

      <h2>YouTube + social examples</h2>
      <CodeBlock language="python" filename="extra_endpoints.py" code={`import requests

# YouTube search
yt = requests.post(
    "https://api.fetchium.com/v1/youtube/search",
    headers=HEADERS,
    json={"query": "Java learning", "max_results": 10},
    timeout=30,
)
yt.raise_for_status()

# Social research
social = requests.post(
    "https://api.fetchium.com/v1/social/research",
    headers=HEADERS,
    json={
        "query": "rust vs go",
        "platforms": ["reddit", "hackernews"],
        "max_per_platform": 20,
    },
    timeout=30,
)
social.raise_for_status()`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/sdk/typescript", title: "TypeScript SDK", desc: "TS/Node examples" },
          { href: "/docs/sdk/curl", title: "curl Examples", desc: "Terminal examples" },
          { href: "/docs/api/search", title: "Search API", desc: "Full API reference" },
          { href: "/docs/api/social", title: "Social API", desc: "Social endpoints" },
        ].map((l) => (
          <Link key={l.href} href={l.href} className="glass-card rounded-xl p-4 no-underline group">
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{l.title} →</div>
            <div className="text-xs text-slate-500 mt-1">{l.desc}</div>
          </Link>
        ))}
      </div>
    </article>
  );
}
