# Security Policy

## Supported versions

Fetchium is under active development. Security fixes are applied to the latest released version on
the `main` branch. Please make sure you're on the most recent release before reporting.

| Version | Supported |
|---------|-----------|
| latest `main` / newest release | ✅ |
| older releases | ❌ (please upgrade) |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues, discussions, or pull
requests.**

Instead, report them privately using one of:

1. **GitHub Security Advisories** (preferred) — go to the
   [Security tab](https://github.com/zuhabul/Fetchium/security/advisories/new) and open a private
   advisory. This keeps the report confidential while we work on a fix.
2. **Email** — contact the maintainer at the address on the
   [GitHub profile](https://github.com/zuhabul). Please include `FETCHIUM SECURITY` in the subject.

When reporting, please include:
- A description of the vulnerability and its impact.
- Steps to reproduce (a proof of concept if possible).
- Affected version / commit.
- Any suggested remediation.

## What to expect

- **Acknowledgement** within 72 hours.
- An initial assessment and severity classification.
- Regular updates as we work on a fix.
- Credit in the release notes once the issue is resolved (unless you prefer to remain anonymous).

Please give us a reasonable window to address the issue before any public disclosure. We're a small
project and appreciate coordinated disclosure.

## Scope

Fetchium fetches and processes untrusted content from the open web. Reports of particular interest:
- SSRF or request-forgery via crafted URLs or redirects.
- Sandbox escapes when rendering/extracting hostile pages.
- Injection or deserialization issues in the REST/MCP API surfaces.
- Path traversal or arbitrary write during caching/extraction.
- Leakage of configured API keys/credentials in logs or output.

## Good to know

- Fetchium stores configuration (including any API keys you provide) in `~/.fetchium/config.toml`.
  Keys are **never** committed to the repository. The only hardcoded API constant in the codebase
  is YouTube's *public* InnerTube web-client key, which is a non-secret value shared across all
  YouTube web clients and is used only as a fallback when a live key cannot be extracted.
