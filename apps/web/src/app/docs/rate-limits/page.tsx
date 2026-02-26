import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Rate Limits" };

export default function RateLimits() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Getting Started</div>
      <h1>Rate Limits</h1>
      <p>
        HyperSearchX enforces two types of rate limits: a monthly request quota per plan, and a
        per-minute limit to prevent burst abuse. Both limits are tracked per API key.
      </p>

      <h2>Plan quotas</h2>
      <table>
        <thead><tr><th>Plan</th><th>Requests / month</th><th>Requests / minute</th><th>Price</th></tr></thead>
        <tbody>
          <tr><td>Free</td><td>1,000</td><td>60</td><td>$0</td></tr>
          <tr><td>Starter</td><td>10,000</td><td>200</td><td>$19 / mo</td></tr>
          <tr><td>Pro</td><td>100,000</td><td>500</td><td>$79 / mo</td></tr>
          <tr><td>Enterprise</td><td>Unlimited</td><td>2,000</td><td>Custom</td></tr>
        </tbody>
      </table>

      <div className="callout">
        Monthly quotas reset at midnight UTC on the first day of each calendar month, regardless of
        when you subscribed.
      </div>

      <h2>Rate limit headers</h2>
      <p>
        Every API response includes headers showing your current rate limit status:
      </p>

      <CodeBlock language="text" filename="response-headers.txt" code={`HTTP/2 200 OK
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 847
X-RateLimit-Reset: 2025-07-01T00:00:00Z
X-Request-Id: req_01j8xk3m7p4qr5st6uv7wx8yz`} />

      <table>
        <thead><tr><th>Header</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>X-RateLimit-Limit</code></td><td>Monthly quota for your plan</td></tr>
          <tr><td><code>X-RateLimit-Remaining</code></td><td>Requests remaining this month</td></tr>
          <tr><td><code>X-RateLimit-Reset</code></td><td>ISO 8601 timestamp when quota resets</td></tr>
          <tr><td><code>X-Request-Id</code></td><td>Unique request ID for support/debugging</td></tr>
        </tbody>
      </table>

      <h2>Rate limit exceeded response</h2>
      <p>
        When you exceed the per-minute limit, you receive a <code>429 Too Many Requests</code> response:
      </p>

      <CodeBlock language="json" filename="429-response.json" code={`{
  "error": {
    "code": "rate_limited",
    "message": "Too many requests. You have exceeded the 60 requests/minute limit.",
    "retry_after": 60,
    "request_id": "req_01j8xk3..."
  }
}`} />

      <CodeBlock language="text" filename="429-headers.txt" code={`HTTP/2 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 1000
X-RateLimit-Reset: 2025-07-01T00:00:00Z`} />

      <p>
        When the monthly quota is exhausted, you receive a <code>429</code> with code{" "}
        <code>quota_exceeded</code>:
      </p>

      <CodeBlock language="json" code={`{
  "error": {
    "code": "quota_exceeded",
    "message": "Monthly quota exhausted. Resets 2025-07-01T00:00:00Z or upgrade your plan.",
    "request_id": "req_01j8xk3..."
  }
}`} />

      <h2>Handling rate limits in code</h2>

      <CodeBlock language="typescript" filename="retry.ts" code={`async function searchWithRetry(query: string, maxRetries = 3): Promise<any> {
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    const res = await fetch("https://api.hypersearchx.zuhabul.com/v1/search", {
      method: "POST",
      headers: {
        "Authorization": \`Bearer \${process.env.HSX_API_KEY}\`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ query, tier: "summary" }),
    });

    if (res.status === 429) {
      const retryAfter = parseInt(res.headers.get("Retry-After") ?? "60", 10);
      if (attempt < maxRetries) {
        await new Promise(r => setTimeout(r, retryAfter * 1000));
        continue;
      }
    }

    if (!res.ok) throw new Error(\`API error: \${res.status}\`);
    return res.json();
  }
  throw new Error("Max retries exceeded");
}`} />

      <CodeBlock language="python" filename="retry.py" code={`import time, requests, os

HSX_BASE = "https://api.hypersearchx.zuhabul.com"
HEADERS = {"Authorization": f"Bearer {os.environ['HSX_API_KEY']}"}

def search_with_retry(query: str, max_retries: int = 3) -> dict:
    for attempt in range(max_retries + 1):
        r = requests.post(f"{HSX_BASE}/v1/search",
            headers=HEADERS,
            json={"query": query, "tier": "summary"})

        if r.status_code == 429:
            retry_after = int(r.headers.get("Retry-After", 60))
            if attempt < max_retries:
                time.sleep(retry_after)
                continue

        r.raise_for_status()
        return r.json()
    raise RuntimeError("Max retries exceeded")`} />

      <h2>IP-level rate limiting</h2>
      <p>
        In addition to per-key limits, the API gateway enforces IP-level rate limiting of 100
        requests/second average with a burst of 200. This protects against DDoS and is independent
        of your API key quota.
      </p>

      <h2>Monitoring usage</h2>
      <p>
        Check your current usage via the <code>/v1/usage</code> endpoint:
      </p>

      <CodeBlock language="bash" code={`curl https://api.hypersearchx.zuhabul.com/v1/usage \\
  -H "Authorization: Bearer hsx_your_key"`} />

      <CodeBlock language="json" filename="usage-response.json" code={`{
  "plan": "free",
  "quota": 1000,
  "used": 153,
  "remaining": 847,
  "reset_at": "2025-07-01T00:00:00Z",
  "per_minute_limit": 60,
  "endpoints": {
    "search": 120,
    "research": 28,
    "scrape": 5
  }
}`} />

      <p>
        You can also view real-time usage in the{" "}
        <Link href="https://app.hypersearchx.zuhabul.com">dashboard</Link>.
      </p>

      <h2>Upgrade your plan</h2>
      <p>
        Upgrade anytime from the{" "}
        <Link href="https://app.hypersearchx.zuhabul.com">dashboard → Billing</Link>. Quota
        increases take effect immediately.
      </p>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "/docs/api/search", title: "Search API", desc: "Full parameter reference" },
          { href: "/docs/authentication", title: "Authentication", desc: "Key management and rotation" },
          { href: "/docs/errors", title: "Error Codes", desc: "All error responses documented" },
          { href: "/docs/api/usage", title: "Usage API", desc: "Programmatic quota monitoring" },
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
