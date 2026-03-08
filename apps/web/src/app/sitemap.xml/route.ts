import { headers } from "next/headers";

const MAIN_SITE_URL = "https://fetchium.com";
const DOCS_SITE_URL = "https://docs.fetchium.com";

const mainRoutes = [
  "",
  "/about",
  "/blog",
  "/blog/how-to-build-rag-pipeline-fetchium",
  "/blog/token-budgeted-extraction-llm-cost",
  "/changelog",
  "/compare/exa",
  "/compare/firecrawl",
  "/compare/perplexity",
  "/compare/serpapi",
  "/compare/tavily",
  "/contact",
  "/cookies",
  "/pricing",
  "/privacy",
  "/product/extract",
  "/product/mcp",
  "/product/research",
  "/product/search",
  "/roadmap",
  "/security",
  "/status",
  "/terms",
];

const docsRoutes = [
  "",
  "/algorithms",
  "/algorithms/cep",
  "/algorithms/hyperfusion",
  "/algorithms/qatbe",
  "/algorithms/spre",
  "/api/admin-keys",
  "/api/estimate",
  "/api/health",
  "/api/research",
  "/api/scrape",
  "/api/search",
  "/api/social",
  "/api/usage",
  "/api/youtube",
  "/authentication",
  "/errors",
  "/quickstart",
  "/rate-limits",
  "/sdk/curl",
  "/sdk/mcp",
  "/sdk/python",
  "/sdk/typescript",
  "/self-hosting/config",
  "/self-hosting/docker",
  "/self-hosting/searxng",
];

function resolveHost(hostHeader: string | null): string {
  return hostHeader?.split(":")[0].toLowerCase() ?? "";
}

function renderUrlset(baseUrl: string, routes: string[]) {
  const lastModified = new Date().toISOString();
  const urls = routes
    .map((route) => {
      const url = `${baseUrl}${route}`;
      const priority = route === "" ? "1.0" : "0.7";
      const changeFrequency = route === "" ? "weekly" : "monthly";

      return [
        "  <url>",
        `    <loc>${url}</loc>`,
        `    <lastmod>${lastModified}</lastmod>`,
        `    <changefreq>${changeFrequency}</changefreq>`,
        `    <priority>${priority}</priority>`,
        "  </url>",
      ].join("\n");
    })
    .join("\n");

  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    urls,
    "</urlset>",
    "",
  ].join("\n");
}

export async function GET() {
  const requestHeaders = await headers();
  const host = resolveHost(
    requestHeaders.get("x-forwarded-host") ?? requestHeaders.get("host"),
  );
  const isDocsHost = host === "docs.fetchium.com";

  const xml = isDocsHost
    ? renderUrlset(DOCS_SITE_URL, docsRoutes)
    : renderUrlset(MAIN_SITE_URL, mainRoutes);

  return new Response(xml, {
    headers: {
      "Content-Type": "application/xml; charset=utf-8",
      "Cache-Control": "public, s-maxage=3600, stale-while-revalidate=86400",
    },
  });
}
