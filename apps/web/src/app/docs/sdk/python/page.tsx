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
        Use Fetchium from Python with <code>requests</code> or <code>httpx</code>.
        No SDK package required — the API is straightforward to call directly.
      </p>

      <h2>Quickstart</h2>

      <CodeBlock language="python" filename="fetchium_client.py" code={`import os
import requests

FETCHIUM_BASE = "https://api.fetchium.com"
FETCHIUM_KEY = os.environ["FETCHIUM_API_KEY"]

HEADERS = {
    "Authorization": f"Bearer {FETCHIUM_KEY}",
    "Content-Type": "application/json",
}


def search(query: str, tier: str = "summary") -> dict:
    """Search the web with HyperFusion ranking."""
    r = requests.post(
        f"{FETCHIUM_BASE}/v1/search",
        headers=HEADERS,
        json={"query": query, "tier": tier},
    )
    r.raise_for_status()
    return r.json()


def scrape(url: str, tier: str = "summary") -> dict:
    """Extract content from a URL using CEP."""
    r = requests.post(
        f"{FETCHIUM_BASE}/v1/scrape",
        headers=HEADERS,
        json={"url": url, "tier": tier},
    )
    r.raise_for_status()
    return r.json()


def research(query: str, depth: str = "thorough") -> dict:
    """Run a deep multi-source research query."""
    r = requests.post(
        f"{FETCHIUM_BASE}/v1/research",
        headers=HEADERS,
        json={"query": query, "depth": depth},
        timeout=120,  # research can take up to 60s
    )
    r.raise_for_status()
    return r.json()


if __name__ == "__main__":
    data = search("rust async programming best practices", tier="detailed")
    for result in data["results"]:
        print(f"{result['score']:.2f}  {result['title']}")
        print(f"       {result['url']}")
        print()`} />

      <h2>Async with httpx</h2>

      <CodeBlock language="python" filename="fetchium_async.py" code={`import asyncio
import os
import httpx

FETCHIUM_BASE = "https://api.fetchium.com"
FETCHIUM_KEY = os.environ["FETCHIUM_API_KEY"]

HEADERS = {
    "Authorization": f"Bearer {FETCHIUM_KEY}",
    "Content-Type": "application/json",
}


async def search_multiple(queries: list[str]) -> list[dict]:
    """Search multiple queries in parallel."""
    async with httpx.AsyncClient(headers=HEADERS, timeout=30) as client:
        tasks = [
            client.post(f"{FETCHIUM_BASE}/v1/search", json={"query": q, "tier": "summary"})
            for q in queries
        ]
        responses = await asyncio.gather(*tasks)
        return [r.raise_for_status().json() for r in responses]


async def main():
    results = await search_multiple([
        "python async best practices",
        "fastapi vs django 2025",
        "uv package manager rust python",
    ])
    for query, data in zip(["python async", "fastapi vs django", "uv manager"], results):
        print(f"\\n=== {query} ===")
        for r in data["results"][:3]:
            print(f"  • {r['title'][:80]}")


asyncio.run(main())`} />

      <h2>LangChain tool</h2>

      <CodeBlock language="python" filename="langchain_tool.py" code={`import os
import requests
from langchain.tools import Tool


def fetchium_search(query: str) -> str:
    """Search Fetchium and return formatted results."""
    r = requests.post(
        "https://api.fetchium.com/v1/search",
        headers={"Authorization": f"Bearer {os.environ['FETCHIUM_API_KEY']}"},
        json={"query": query, "tier": "summary", "max_sources": 5},
    )
    r.raise_for_status()
    results = r.json()["results"]
    return "\\n\\n".join(
        f"**{res['title']}** ({res['url']})\\n{res['snippet']}"
        for res in results
    )


web_search_tool = Tool(
    name="web_search",
    description="Search the web for current information on any topic",
    func=fetchium_search,
)`} />

      <h2>RAG pipeline example</h2>

      <CodeBlock language="python" filename="rag_pipeline.py" code={`import os
import requests
from anthropic import Anthropic

client = Anthropic()
FETCHIUM_KEY = os.environ["FETCHIUM_API_KEY"]


def search_and_answer(question: str) -> str:
    # 1. Search for relevant content
    search_res = requests.post(
        "https://api.fetchium.com/v1/search",
        headers={"Authorization": f"Bearer {FETCHIUM_KEY}"},
        json={"query": question, "tier": "detailed", "max_sources": 6},
    )
    search_res.raise_for_status()
    results = search_res.json()["results"]

    # 2. Build context from top results
    context = "\\n\\n".join(
        f"Source: {r['url']}\\nTitle: {r['title']}\\n{r['snippet']}"
        for r in results[:5]
    )

    # 3. Answer with Claude
    message = client.messages.create(
        model="claude-opus-4-6",
        max_tokens=1024,
        messages=[{
            "role": "user",
            "content": f"Based on these search results, answer: {question}\\n\\n{context}"
        }]
    )
    return message.content[0].text


print(search_and_answer("What are the best practices for Rust error handling in 2025?"))`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/sdk/typescript", title: "TypeScript SDK", desc: "JavaScript/TS examples" },
          { href: "/docs/sdk/curl", title: "curl Examples", desc: "CLI / shell usage" },
          { href: "/docs/api/search", title: "Search API", desc: "Full parameter reference" },
          { href: "/docs/api/research", title: "Research API", desc: "Deep research queries" },
        ].map(l => (
          <Link key={l.href} href={l.href} className="glass-card rounded-xl p-4 no-underline group">
            <div className="font-medium text-slate-200 text-sm group-hover:text-indigo-300 transition-colors">{l.title} →</div>
            <div className="text-xs text-slate-500 mt-1">{l.desc}</div>
          </Link>
        ))}
      </div>
    </article>
  );
}
