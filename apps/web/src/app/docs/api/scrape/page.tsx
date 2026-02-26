import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Scrape API Reference" };

export default function ScrapeApiReference() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">API Reference</div>
      <h1>Scrape API</h1>
      <p>
        Extract clean, structured content from any URL using the 5-layer CEP (Content Extraction
        Protocol) pipeline. Unlike basic scrapers, CEP handles JavaScript-rendered pages, PDFs, and
        complex layouts while applying QATBE token-budgeted extraction.
      </p>

      <div className="flex items-center gap-3 my-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] font-mono text-sm not-prose">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-semibold border bg-indigo-500/15 text-indigo-300 border-indigo-500/30">POST</span>
        <span className="text-slate-300">/v1/scrape</span>
      </div>

      <h2>CEP pipeline layers</h2>
      <p>
        The Content Extraction Protocol tries each layer in order, falling back when the previous
        layer fails or produces insufficient content:
      </p>
      <table>
        <thead><tr><th>Layer</th><th>Method</th><th>Best for</th></tr></thead>
        <tbody>
          <tr><td>1</td><td>CSS selector extraction</td><td>Well-structured HTML with known schemas</td></tr>
          <tr><td>2</td><td>Readability algorithm</td><td>Article pages, blogs, documentation</td></tr>
          <tr><td>3</td><td>Headless JS rendering</td><td>SPAs, React/Vue/Angular apps</td></tr>
          <tr><td>4</td><td>PDF text extraction</td><td>PDF files, academic papers</td></tr>
          <tr><td>5</td><td>Screenshot OCR</td><td>Image-heavy pages, canvas rendering</td></tr>
        </tbody>
      </table>

      <h2>Request parameters</h2>
      <table>
        <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>url</code></td><td>string</td><td>Yes</td><td>URL to scrape. Must be a valid HTTP(S) URL.</td></tr>
          <tr><td><code>tier</code></td><td>string</td><td>No</td><td>Detail level (same as search). Default: <code>summary</code>.</td></tr>
          <tr><td><code>token_budget</code></td><td>integer</td><td>No</td><td>Override tier. Range: 100–50,000.</td></tr>
          <tr><td><code>query</code></td><td>string</td><td>No</td><td>Query to focus extraction on (boosts query-relevant segments via QATBE).</td></tr>
          <tr><td><code>include_metadata</code></td><td>boolean</td><td>No</td><td>Include page metadata (title, description, OG tags). Default: true.</td></tr>
          <tr><td><code>include_links</code></td><td>boolean</td><td>No</td><td>Include extracted internal/external links. Default: false.</td></tr>
          <tr><td><code>wait_for_js</code></td><td>boolean</td><td>No</td><td>Force headless JS rendering even for static pages. Default: false.</td></tr>
        </tbody>
      </table>

      <h2>Example request</h2>

      <CodeBlock language="bash" filename="scrape.sh" code={`curl -X POST https://api.hypersearchx.zuhabul.com/v1/scrape \\
  -H "Authorization: Bearer hsx_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://tokio.rs/tokio/tutorial/hello-tokio",
    "tier": "detailed",
    "query": "async runtime tutorial"
  }'`} />

      <h2>Response</h2>

      <CodeBlock language="json" filename="response.json" code={`{
  "meta": {
    "url": "https://tokio.rs/tokio/tutorial/hello-tokio",
    "title": "Hello Tokio — Tokio",
    "description": "A walkthrough of the Hello Tokio application",
    "tokens_used": 3210,
    "duration_ms": 892,
    "cep_layer_used": 2,
    "result_id": "f1e2d3c4-..."
  },
  "content": "Tokio is an asynchronous runtime for the Rust programming language...",
  "segments": [
    {
      "type": "heading",
      "text": "Hello Tokio",
      "level": 1,
      "token_count": 2
    },
    {
      "type": "paragraph",
      "text": "We will start by writing a very basic Tokio application...",
      "token_count": 48
    },
    {
      "type": "code",
      "text": "#[tokio::main]\\nasync fn main() {\\n    println!(\\\"Hello, Tokio!\\\");\\n}",
      "language": "rust",
      "token_count": 22
    }
  ]
}`} />

      <h3>Response fields</h3>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>meta.cep_layer_used</code></td><td>integer</td><td>Which CEP layer (1–5) produced the content</td></tr>
          <tr><td><code>content</code></td><td>string</td><td>Clean extracted text within token budget</td></tr>
          <tr><td><code>segments</code></td><td>array</td><td>SCS-segmented content blocks with type and token count</td></tr>
          <tr><td><code>segments[].type</code></td><td>string</td><td>
            <code>heading</code>, <code>paragraph</code>, <code>code</code>, <code>list</code>,{" "}
            <code>table</code>, <code>quote</code>, <code>metadata</code>, <code>other</code>
          </td></tr>
        </tbody>
      </table>

      <h2>PDF extraction</h2>
      <p>Point the scrape endpoint at any PDF URL — CEP layer 4 handles extraction automatically:</p>
      <CodeBlock language="bash" code={`curl -X POST https://api.hypersearchx.zuhabul.com/v1/scrape \\
  -H "Authorization: Bearer hsx_your_key" \\
  -H "Content-Type: application/json" \\
  -d '{"url": "https://arxiv.org/pdf/2301.00234.pdf", "tier": "detailed"}'`} />

      <h2>JavaScript-rendered pages</h2>
      <p>
        CEP automatically detects and uses headless rendering for SPAs. For pages where static
        extraction is insufficient, force JS rendering:
      </p>
      <CodeBlock language="json" code={`{
  "url": "https://app.example.com/dashboard",
  "wait_for_js": true,
  "tier": "summary"
}`} />

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/search", title: "Search API", desc: "Federated multi-backend search" },
          { href: "/docs/api/research", title: "Research API", desc: "Deep multi-source research" },
          { href: "/docs/algorithms/cep", title: "CEP Algorithm", desc: "How content extraction works" },
          { href: "/docs/algorithms/qatbe", title: "QATBE Algorithm", desc: "Token budget extraction" },
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
