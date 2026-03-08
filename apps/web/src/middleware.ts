import type { NextRequest } from "next/server";
import { NextResponse } from "next/server";

const MAIN_HOSTS = new Set(["fetchium.com", "www.fetchium.com"]);
const DOCS_HOSTS = new Set(["docs.fetchium.com"]);
const EXCLUDED_PREFIXES = ["/_next", "/images"];
const EXCLUDED_PATHS = ["/favicon.ico", "/robots.txt", "/sitemap.xml"];

function normalizeHost(host: string): string {
  return host.split(":")[0].toLowerCase();
}

function shouldBypass(pathname: string): boolean {
  return (
    EXCLUDED_PATHS.includes(pathname) ||
    EXCLUDED_PREFIXES.some((prefix) => pathname.startsWith(prefix))
  );
}

export function middleware(request: NextRequest) {
  const host = normalizeHost(request.headers.get("host") ?? "");
  const { pathname, search } = request.nextUrl;

  if (shouldBypass(pathname)) {
    return NextResponse.next();
  }

  if (DOCS_HOSTS.has(host)) {
    const docsPath = pathname === "/" ? "/docs" : `/docs${pathname}`;
    const rewriteUrl = request.nextUrl.clone();

    rewriteUrl.pathname = docsPath;
    return NextResponse.rewrite(rewriteUrl);
  }

  if (MAIN_HOSTS.has(host) && (pathname === "/docs" || pathname.startsWith("/docs/"))) {
    const redirectUrl = new URL("https://docs.fetchium.com");
    const docsPath = pathname === "/docs" ? "/" : pathname.replace(/^\/docs/, "");

    redirectUrl.pathname = docsPath;
    redirectUrl.search = search;

    return NextResponse.redirect(redirectUrl, 308);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!.*\\..*).*)", "/", "/robots.txt", "/sitemap.xml"],
};
