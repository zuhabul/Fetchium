import type { Metadata } from "next";
import Link from "next/link";
import CodeBlock from "@/components/docs/CodeBlock";

export const metadata: Metadata = { title: "Authentication" };

export default function Authentication() {
  return (
    <article className="docs-content max-w-3xl">
      <div className="text-xs text-slate-500 mb-2 font-mono">Getting Started</div>
      <h1>Authentication</h1>
      <p>
        Fetchium uses API keys for authentication. All requests (except{" "}
        <code>/health</code>) require a valid API key in the{" "}
        <code>Authorization</code> header.
      </p>

      <h2>API key format</h2>
      <p>
        API keys are prefixed with <code>fetchium_</code> followed by 64 lowercase hexadecimal
        characters, giving 256 bits of entropy:
      </p>
      <CodeBlock language="text" code={`fetchium_4626d3fc3fd6693aaaf2d8f5fd084a71320f48e0e94904ff9dc9a1064b4bb998`} />

      <h2>Using your API key</h2>
      <p>
        Pass the key as a <code>Bearer</code> token in the <code>Authorization</code> header on every
        request:
      </p>

      <CodeBlock language="bash" code={`curl -X POST ***REMOVED***/v1/search \\
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query": "rust async programming", "tier": "summary"}'`} />

      <div className="callout">
        <strong>Security:</strong> Never expose your API key in client-side code, public repositories,
        or logs. Always load it from environment variables.
      </div>

      <h2>Getting an API key</h2>
      <ol>
        <li>Sign up at <Link href="https://app.fetchium.com">app.fetchium.com</Link></li>
        <li>Navigate to <strong>API Keys</strong> in the sidebar</li>
        <li>Click <strong>Create Key</strong></li>
        <li>Copy the key immediately — it is shown only once</li>
      </ol>

      <div className="callout">
        <strong>One-time display:</strong> The full key is shown <strong>only at creation time</strong>.
        After you navigate away, only the last 8 characters are shown. Store the key in a password
        manager or secrets vault before closing the dialog.
      </div>

      <h2>Key management</h2>
      <p>
        End-user keys are typically created in the dashboard. The REST admin endpoints for
        creating, listing, and revoking keys require <code>X-Admin-Secret</code> and are documented
        separately in the admin reference.
      </p>

      <h2>Error responses</h2>
      <table>
        <thead><tr><th>Status</th><th>Code</th><th>Cause</th></tr></thead>
        <tbody>
          <tr><td><code>401</code></td><td><code>missing_token</code></td><td>No Bearer token was provided</td></tr>
          <tr><td><code>401</code></td><td><code>invalid_token</code></td><td>Malformed, invalid, or revoked API key</td></tr>
          <tr><td><code>429</code></td><td><code>rate_limited</code></td><td>Quota or per-minute limit exceeded</td></tr>
        </tbody>
      </table>

      <CodeBlock language="json" filename="401-response.json" code={`{
  "type": "https://docs.fetchium.com/errors/invalid_token",
  "title": "Invalid or revoked API key",
  "status": 401
}`} />

      <h2>Environment variable setup</h2>
      <CodeBlock language="bash" filename=".env" code={`# Add to your .env or secrets manager
FETCHIUM_API_KEY=fetchium_4626d3fc3fd6693aaaf2d8f5fd084a71...`} />

      <CodeBlock language="typescript" filename="client.ts" code={`const FETCHIUM_API_KEY = process.env.FETCHIUM_API_KEY;
if (!FETCHIUM_API_KEY) throw new Error("FETCHIUM_API_KEY is not set");

const headers = {
  "Authorization": \`Bearer \${FETCHIUM_API_KEY}\`,
  "Content-Type": "application/json",
};`} />

      <CodeBlock language="python" filename="client.py" code={`import os

FETCHIUM_API_KEY = os.environ["FETCHIUM_API_KEY"]  # raises KeyError if missing
HEADERS = {
    "Authorization": f"Bearer {FETCHIUM_API_KEY}",
    "Content-Type": "application/json",
}`} />

      <h2>Key rotation</h2>
      <p>
        Rotate keys regularly and immediately if compromised. The recommended workflow:
      </p>
      <ol>
        <li>Create a new key in the dashboard</li>
        <li>Deploy the new key to your environment</li>
        <li>Verify requests are working with the new key</li>
        <li>Revoke the old key</li>
      </ol>

      <h2>Next steps</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 not-prose">
        {[
          { href: "https://docs.fetchium.com/quickstart", title: "Quick Start", desc: "Make your first API call" },
          { href: "https://docs.fetchium.com/rate-limits", title: "Rate Limits", desc: "Understand quotas and headers" },
          { href: "https://docs.fetchium.com/api/search", title: "Search API", desc: "Full search reference" },
          { href: "https://docs.fetchium.com/api/admin-keys", title: "Admin Keys", desc: "Create, list, and revoke keys with X-Admin-Secret" },
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
