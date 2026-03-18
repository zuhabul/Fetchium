export type DashboardEndpoint = {
  path: string;
  label: string;
  method: "GET" | "POST";
  category: "Core API" | "Research" | "Media" | "Social" | "Async Jobs";
  description: string;
  docsHref: string;
  sampleBody?: string;
  sampleCurl: string;
  playgroundSupported?: boolean;
  asyncVariant?: string;
  pollingRoute?: string;
  requiresPathParam?: "id";
  supportsBody?: boolean;
};

const searchBody = JSON.stringify(
  {
    query: "rust async programming",
    max_sources: 5,
    tier: "summary",
  },
  null,
  2,
);

const scrapeBody = JSON.stringify(
  {
    url: "https://doc.rust-lang.org/book/",
    query: "ownership rules",
  },
  null,
  2,
);

const fetchBody = JSON.stringify(
  {
    url: "https://doc.rust-lang.org/book/",
    token_budget: 3000,
  },
  null,
  2,
);

const researchBody = JSON.stringify(
  {
    query: "best practices for LLM agents in 2025",
    token_budget: 4000,
  },
  null,
  2,
);

const estimateBody = JSON.stringify(
  {
    url: "https://doc.rust-lang.org/book/",
  },
  null,
  2,
);

const youtubeSearchBody = JSON.stringify(
  {
    query: "rust programming tutorial",
    max_results: 5,
  },
  null,
  2,
);

const youtubeAnalyzeBody = JSON.stringify(
  {
    url: "https://www.youtube.com/watch?v=PkZNo7MFNFg",
  },
  null,
  2,
);

const redditBody = JSON.stringify(
  {
    query: "rustlang",
    max_results: 10,
  },
  null,
  2,
);

const socialResearchBody = JSON.stringify(
  {
    query: "best developer tools for rust teams",
    platforms: ["reddit", "hackernews"],
    max_per_platform: 5,
  },
  null,
  2,
);

const hackerNewsBody = JSON.stringify(
  {
    query: "rust async",
    max_results: 10,
  },
  null,
  2,
);

export const dashboardEndpoints: DashboardEndpoint[] = [
  {
    path: "/v1/search",
    label: "Search",
    method: "POST",
    category: "Core API",
    description: "Federated search across retrieval backends with ranking and citations.",
    docsHref: "https://docs.fetchium.com/api/search",
    sampleBody: searchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/search \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${searchBody}'`,
    playgroundSupported: true,
  },
  {
    path: "/v1/scrape",
    label: "Scrape",
    method: "POST",
    category: "Core API",
    description: "Fetch and extract a page with query-aware structured content output.",
    docsHref: "https://docs.fetchium.com/api/scrape",
    sampleBody: scrapeBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/scrape \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${scrapeBody}'`,
    playgroundSupported: true,
  },
  {
    path: "/v1/fetch",
    label: "Fetch",
    method: "POST",
    category: "Core API",
    description: "Extract clean content from a single URL with a token budget.",
    docsHref: "https://docs.fetchium.com/api/scrape",
    sampleBody: fetchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/fetch \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${fetchBody}'`,
    playgroundSupported: true,
  },
  {
    path: "/v1/estimate",
    label: "Estimate",
    method: "POST",
    category: "Core API",
    description: "Preview extraction cost and token budget before fetching.",
    docsHref: "https://docs.fetchium.com/api/estimate",
    sampleBody: estimateBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/estimate \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${estimateBody}'`,
    playgroundSupported: true,
  },
  {
    path: "/v1/research",
    label: "Research",
    method: "POST",
    category: "Research",
    description: "Multi-step research synthesis with source-backed output.",
    docsHref: "https://docs.fetchium.com/api/research",
    sampleBody: researchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/research \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${researchBody}'`,
    playgroundSupported: true,
    asyncVariant: "/v1/research/jobs",
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/research/jobs",
    label: "Research Jobs",
    method: "POST",
    category: "Async Jobs",
    description: "Submit a research job for asynchronous execution.",
    docsHref: "https://docs.fetchium.com/api/async-jobs",
    sampleBody: researchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/research/jobs \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${researchBody}'`,
    playgroundSupported: true,
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/youtube/search",
    label: "YouTube Search",
    method: "POST",
    category: "Media",
    description: "Search YouTube content with Fetchium's media pipeline.",
    docsHref: "https://docs.fetchium.com/api/youtube",
    sampleBody: youtubeSearchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/youtube/search \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${youtubeSearchBody}'`,
    playgroundSupported: true,
    asyncVariant: "/v1/youtube/search/jobs",
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/youtube/search/jobs",
    label: "YouTube Search Jobs",
    method: "POST",
    category: "Async Jobs",
    description: "Submit a YouTube search job for asynchronous execution.",
    docsHref: "https://docs.fetchium.com/api/async-jobs",
    sampleBody: youtubeSearchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/youtube/search/jobs \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${youtubeSearchBody}'`,
    playgroundSupported: true,
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/youtube/analyze",
    label: "YouTube Analyze",
    method: "POST",
    category: "Media",
    description: "Analyze a single video URL for metadata and transcript intelligence.",
    docsHref: "https://docs.fetchium.com/api/youtube",
    sampleBody: youtubeAnalyzeBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/youtube/analyze \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${youtubeAnalyzeBody}'`,
    playgroundSupported: true,
    asyncVariant: "/v1/youtube/analyze/jobs",
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/youtube/analyze/jobs",
    label: "YouTube Analyze Jobs",
    method: "POST",
    category: "Async Jobs",
    description: "Submit a YouTube analysis job for asynchronous execution.",
    docsHref: "https://docs.fetchium.com/api/async-jobs",
    sampleBody: youtubeAnalyzeBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/youtube/analyze/jobs \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${youtubeAnalyzeBody}'`,
    playgroundSupported: true,
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/social/research",
    label: "Social Research",
    method: "POST",
    category: "Social",
    description: "Cross-platform social research across the supported social sources.",
    docsHref: "https://docs.fetchium.com/api/social",
    sampleBody: socialResearchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/social/research \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${socialResearchBody}'`,
    playgroundSupported: true,
    asyncVariant: "/v1/social/research/jobs",
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/social/research/jobs",
    label: "Social Research Jobs",
    method: "POST",
    category: "Async Jobs",
    description: "Submit a social research job for asynchronous execution.",
    docsHref: "https://docs.fetchium.com/api/async-jobs",
    sampleBody: socialResearchBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/social/research/jobs \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${socialResearchBody}'`,
    playgroundSupported: true,
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/social/reddit",
    label: "Reddit Social",
    method: "POST",
    category: "Social",
    description: "Query social signals from Reddit through the Fetchium API.",
    docsHref: "https://docs.fetchium.com/api/social",
    sampleBody: redditBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/social/reddit \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${redditBody}'`,
    playgroundSupported: true,
    asyncVariant: "/v1/social/reddit/jobs",
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/social/reddit/jobs",
    label: "Reddit Jobs",
    method: "POST",
    category: "Async Jobs",
    description: "Submit a Reddit search job for asynchronous execution.",
    docsHref: "https://docs.fetchium.com/api/async-jobs",
    sampleBody: redditBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/social/reddit/jobs \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${redditBody}'`,
    playgroundSupported: true,
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/social/hackernews",
    label: "Hacker News Social",
    method: "POST",
    category: "Social",
    description: "Query discussion and signal data from Hacker News.",
    docsHref: "https://docs.fetchium.com/api/social",
    sampleBody: hackerNewsBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/social/hackernews \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${hackerNewsBody}'`,
    playgroundSupported: true,
    asyncVariant: "/v1/social/hackernews/jobs",
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/social/hackernews/jobs",
    label: "Hacker News Jobs",
    method: "POST",
    category: "Async Jobs",
    description: "Submit a Hacker News search job for asynchronous execution.",
    docsHref: "https://docs.fetchium.com/api/async-jobs",
    sampleBody: hackerNewsBody,
    sampleCurl:
      `curl -X POST ***REMOVED***/v1/social/hackernews/jobs \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY" \\\n` +
      `  -H "Content-Type: application/json" \\\n` +
      `  -d '${hackerNewsBody}'`,
    playgroundSupported: true,
    pollingRoute: "/v1/jobs/:id",
  },
  {
    path: "/v1/jobs/:id",
    label: "Job Status",
    method: "GET",
    category: "Async Jobs",
    description: "Poll a previously submitted async job by job ID.",
    docsHref: "https://docs.fetchium.com/api/async-jobs",
    sampleCurl:
      `curl ***REMOVED***/v1/jobs/JOB_ID \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY"`,
    playgroundSupported: true,
    pollingRoute: "/v1/jobs/:id",
    requiresPathParam: "id",
    supportsBody: false,
  },
  {
    path: "/v1/usage",
    label: "Usage",
    method: "GET",
    category: "Core API",
    description: "Inspect request quota, plan limits, and current key usage.",
    docsHref: "https://docs.fetchium.com/api/usage",
    sampleCurl:
      `curl ***REMOVED***/v1/usage \\\n` +
      `  -H "Authorization: Bearer fetchium_YOUR_KEY"`,
  },
];

export const playgroundEndpoints = dashboardEndpoints.filter(
  (endpoint) => endpoint.playgroundSupported,
);

export const playgroundEndpointPaths = new Set(
  playgroundEndpoints.map((endpoint) => endpoint.path),
);

export function endpointByPath(path: string): DashboardEndpoint | undefined {
  return dashboardEndpoints.find((endpoint) => endpoint.path === path);
}

export function resolveEndpointPath(
  endpoint: DashboardEndpoint,
  pathParams?: Record<string, string>,
): string | null {
  if (!endpoint.requiresPathParam) return endpoint.path;
  const value = pathParams?.[endpoint.requiresPathParam]?.trim();
  if (!value) return null;
  return endpoint.path.replace(`:${endpoint.requiresPathParam}`, encodeURIComponent(value));
}
