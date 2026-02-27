import type { Metadata } from "next";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Error Reference" };

export default function Errors() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Reference</div>
      <h1>Error Reference</h1>
      <p>
        All Fetchium API errors follow a consistent JSON format with machine-readable error
        codes, human-readable messages, and a unique request ID for support.
      </p>

      <h2>Error format</h2>
      <CodeBlock language="json" code={`{
  "error": {
    "code": "invalid_request",
    "message": "Human-readable description of what went wrong.",
    "field": "query",
    "request_id": "req_01j8xk3m7p4qr5st6uv7wx8yz"
  }
}`} />

      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>code</code></td><td>string</td><td>Machine-readable error identifier</td></tr>
          <tr><td><code>message</code></td><td>string</td><td>Human-readable explanation</td></tr>
          <tr><td><code>field</code></td><td>string?</td><td>Request field that caused the error (validation errors)</td></tr>
          <tr><td><code>request_id</code></td><td>string</td><td>Unique ID — include when contacting support</td></tr>
        </tbody>
      </table>

      <h2>HTTP status codes</h2>
      <table>
        <thead><tr><th>Status</th><th>Meaning</th></tr></thead>
        <tbody>
          <tr><td><code>200</code></td><td>Success</td></tr>
          <tr><td><code>400</code></td><td>Bad request — validation error in request body</td></tr>
          <tr><td><code>401</code></td><td>Unauthorized — missing or invalid API key</td></tr>
          <tr><td><code>403</code></td><td>Forbidden — key lacks required scope</td></tr>
          <tr><td><code>404</code></td><td>Not found — endpoint does not exist</td></tr>
          <tr><td><code>422</code></td><td>Unprocessable — request parsed but semantically invalid</td></tr>
          <tr><td><code>429</code></td><td>Rate limited — per-minute or monthly quota exceeded</td></tr>
          <tr><td><code>500</code></td><td>Internal server error</td></tr>
          <tr><td><code>503</code></td><td>Service unavailable — upstream dependency down</td></tr>
        </tbody>
      </table>

      <h2>Error codes</h2>

      <h3>Authentication errors (401)</h3>
      <table>
        <thead><tr><th>Code</th><th>Cause</th></tr></thead>
        <tbody>
          <tr><td><code>missing_auth</code></td><td>No Authorization header provided</td></tr>
          <tr><td><code>invalid_auth</code></td><td>Authorization header is not a Bearer token</td></tr>
          <tr><td><code>invalid_key</code></td><td>API key not found or has been revoked</td></tr>
          <tr><td><code>expired_key</code></td><td>API key has passed its expiration date</td></tr>
        </tbody>
      </table>

      <h3>Authorization errors (403)</h3>
      <table>
        <thead><tr><th>Code</th><th>Cause</th></tr></thead>
        <tbody>
          <tr><td><code>insufficient_scope</code></td><td>Key does not have permission for this endpoint</td></tr>
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
          <tr><td><code>upstream_error</code></td><td>A search backend returned an error; results may be partial</td></tr>
          <tr><td><code>timeout</code></td><td>Request exceeded the 60-second processing limit</td></tr>
          <tr><td><code>service_unavailable</code></td><td>Critical dependency is down; check status page</td></tr>
        </tbody>
      </table>

      <h2>Example error handling</h2>
      <CodeBlock language="typescript" filename="error-handling.ts" code={`interface HsxError {
  code: string;
  message: string;
  field?: string;
  request_id: string;
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
    const err: HsxError = body.error;

    switch (err.code) {
      case "rate_limited": {
        const retryAfter = parseInt(res.headers.get("Retry-After") ?? "60", 10);
        throw new Error(\`Rate limited. Retry after \${retryAfter}s\`);
      }
      case "quota_exceeded":
        throw new Error("Monthly quota exhausted. Please upgrade your plan.");
      case "invalid_key":
        throw new Error("Invalid API key. Check FETCHIUM_API_KEY.");
      default:
        throw new Error(\`API error [\${err.code}]: \${err.message} (req: \${err.request_id})\`);
    }
  }

  return res.json();
}`} />
    </article>
  );
}
