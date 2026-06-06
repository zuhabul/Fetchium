import { headers } from "next/headers";

const MAIN_SITE_URL = "https://fetchium.com";
const DOCS_SITE_URL = "https://docs.fetchium.com";

function resolveHost(hostHeader: string | null): string {
  return hostHeader?.split(":")[0].toLowerCase() ?? "";
}

export async function GET() {
  const requestHeaders = await headers();
  const host = resolveHost(
    requestHeaders.get("x-forwarded-host") ?? requestHeaders.get("host"),
  );
  const isDocsHost = host === "docs.fetchium.com";

  const body = isDocsHost
    ? [
        "User-agent: *",
        "Allow: /",
        `Sitemap: ${DOCS_SITE_URL}/sitemap.xml`,
        `Host: ${DOCS_SITE_URL}`,
      ].join("\n")
    : [
        "User-agent: *",
        "Allow: /",
        "Disallow: /docs",
        "Disallow: /docs/",
        `Sitemap: ${MAIN_SITE_URL}/sitemap.xml`,
        `Host: ${MAIN_SITE_URL}`,
      ].join("\n");

  return new Response(`${body}\n`, {
    headers: {
      "Content-Type": "text/plain; charset=utf-8",
      "Cache-Control": "public, s-maxage=3600, stale-while-revalidate=86400",
    },
  });
}
