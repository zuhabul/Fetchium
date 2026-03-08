import type { Metadata } from "next";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Error Reference" };

export default function Errors() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Reference</div>
      <h1>Error Reference</h1>
      <p>
        Fetchium currently exposes two error envelope shapes. Core pipeline handlers return a
        compact JSON error object, while auth and admin middleware return an RFC 7807-style
        problem document.
      </p>

      <h2>Core pipeline error format</h2>
      <CodeBlock language="json" code={`{
  "error": "query cannot be empty",
  "error_type": "invalid_request",
  "status": 400
}`} />

      <h2>Auth / admin error format</h2>
      <CodeBlock language="json" code={`{
  "type": "https://docs.fetchium.com/errors/invalid_token",
  "title": "Invalid or revoked API key",
  "status": 401
}`} />

      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>error_type</code></td><td>string</td><td>Machine-readable identifier on core pipeline errors</td></tr>
          <tr><td><code>error</code></td><td>string</td><td>Human-readable explanation on core pipeline errors</td></tr>
          <tr><td><code>type</code></td><td>string</td><td>Canonical docs URI on auth/admin problem documents</td></tr>
          <tr><td><code>title</code></td><td>string</td><td>Human-readable explanation on problem documents</td></tr>
        </tbody>
      </table>

      <h2>HTTP status codes</h2>
      <table>
        <thead><tr><th>Status</th><th>Meaning</th></tr></thead>
        <tbody>
          <tr><td><code>200</code></td><td>Success</td></tr>
          <tr><td><code>400</code></td><td>Bad request — validation error in request body</td></tr>
          <tr><td><code>401</code></td><td>Unauthorized — missing or invalid API key</td></tr>
          <tr><td><code>404</code></td><td>Not found — endpoint does not exist</td></tr>
          <tr><td><code>429</code></td><td>Rate limited — per-minute or monthly quota exceeded</td></tr>
          <tr><td><code>500</code></td><td>Internal server error</td></tr>
          <tr><td><code>504</code></td><td>Gateway timeout — endpoint exceeded its API timeout</td></tr>
          <tr><td><code>503</code></td><td>Service unavailable — upstream dependency down</td></tr>
        </tbody>
      </table>

      <h2>Error codes</h2>

      <h3>Authentication errors (401)</h3>
      <table>
        <thead><tr><th>Code</th><th>Cause</th></tr></thead>
        <tbody>
          <tr><td><code>missing_token</code></td><td>No Authorization header or Bearer token provided</td></tr>
          <tr><td><code>invalid_token</code></td><td>Authorization token is malformed, invalid, or revoked</td></tr>
        </tbody>
      </table>

      <h3>Validation errors (400)</h3>
      <table>
        <thead><tr><th>Code</th><th>Cause</th></tr></thead>
        <tbody>
          <tr><td><code>invalid_request</code></td><td>Request body validation failed (see <code>field</code>)</td></tr>
          <tr><td><code>query_too_long</code></td><td>Query exceeds 500 characters</td></tr>
          <tr><td><code>invalid_tier</code></td><td>Unknown tier value (must be key_facts, summary, detailed, complete)</td></tr>
          <tr><td><code>invalid_budget</code></td><td>token_budget out of range (100–10,000 for search)</td></tr>
          <tr><td><code>invalid_sources</code></td><td>max_sources out of range (1–20)</td></tr>
          <tr><td><code>invalid_url</code></td><td>Provided URL is not valid or not accessible</td></tr>
          <tr><td><code>payload_too_large</code></td><td>Request body exceeds 1 MB limit</td></tr>
        </tbody>
      </table>

      <h3>Rate limit errors (429)</h3>
      <table>
        <thead><tr><th>Code</th><th>Cause</th></tr></thead>
        <tbody>
          <tr><td><code>rate_limited</code></td><td>Per-minute limit exceeded; wait <code>Retry-After</code> seconds</td></tr>
          <tr><td><code>quota_exceeded</code></td><td>Monthly quota exhausted; upgrade plan or wait for reset</td></tr>
        </tbody>
      </table>

      <h3>Server errors (500 / 503)</h3>
      <table>
        <thead><tr><th>Code</th><th>Cause</th></tr></thead>
        <tbody>
          <tr><td><code>internal_error</code></td><td>Unexpected server error; please retry</td></tr>
          <tr><td><code>search_failed</code></td><td>Search pipeline failed internally</td></tr>
          <tr><td><code>fetch_failed</code></td><td>Fetch/extraction pipeline failed</td></tr>
          <tr><td><code>research_failed</code></td><td>Research pipeline failed internally</td></tr>
          <tr><td><code>request_timeout</code></td><td>Endpoint exceeded the API timeout and should be retried or moved to async jobs</td></tr>
          <tr><td><code>db_error</code></td><td>Auth or usage store failed to respond correctly</td></tr>
        </tbody>
      </table>

      <h2>Example error handling</h2>
      <CodeBlock language="typescript" filename="error-handling.ts" code={`interface HsxError {
  error?: string;
  error_type?: string;
  status: number;
  type?: string;
  title?: string;
}

async function search(query: string) {
  const res = await fetch("https://api.fetchium.com/v1/search", {
    method: "POST",
    headers: {
      "Authorization": \`Bearer \${process.env.FETCHIUM_API_KEY}\`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query, tier: "summary" }),
  });

  if (!res.ok) {
    const body = await res.json();
    const err: HsxError = body;

    switch (err.error_type ?? err.type?.split("/").pop()) {
      case "rate_limited": {
        const retryAfter = parseInt(res.headers.get("Retry-After") ?? "60", 10);
        throw new Error(\`Rate limited. Retry after \${retryAfter}s\`);
      }
      case "quota_exceeded":
        throw new Error("Monthly quota exhausted. Please upgrade your plan.");
      case "invalid_token":
        throw new Error("Invalid API key. Check FETCHIUM_API_KEY.");
      default:
        throw new Error(err.error ?? err.title ?? \`API error \${res.status}\`);
    }
  }

  return res.json();
}`} />
    </article>
  );
}
