# Production Readiness

Fetchium is production-ready only when all of the following are true:

1. The public REST deployment is live and passing the smoke workflow in [.github/workflows/public-smoke.yml](/home/echo/projects/Fetchium/.github/workflows/public-smoke.yml).
2. `PUBLIC_FETCHIUM_API_KEY` and `PUBLIC_FETCHIUM_BASE_URL` are configured in GitHub Actions.
3. If public MCP is offered, `PUBLIC_FETCHIUM_MCP_URL` is configured to the deployed `/mcp` endpoint and the initialize probe passes.
4. `NPM_TOKEN`, `PYPI_API_TOKEN`, and `HOMEBREW_TAP_TOKEN` are configured for release publishing.
5. CI is green across Rust, npm packaging, and Python adapter packaging/tests.

## What Is Implemented

- REST API responses now include a shared `meta` block with `request_id`, `status`, `endpoint`, and `duration_ms`.
- Long-running routes now have async job endpoints under `/v1/*/jobs` plus `/v1/jobs/:id`.
- Release automation publishes:
  - GitHub release binaries
  - npm package `fetchium`
  - Python adapters `fetchium-langchain` and `fetchium-crewai`
  - Homebrew formula
- Deployment automation includes:
  - production compose stack in `infra/docker-compose.prod.yml`
  - rollout script in `scripts/deploy.sh`
  - GitHub Actions deploy workflow in `.github/workflows/deploy.yml`
  - Jenkins pipeline for `server100` in `Jenkinsfile`
- CI validates:
  - Rust fmt/clippy/build/test/docs/coverage
  - npm package syntax and `npm pack --dry-run`
  - Python adapter buildability and tests
- Public smoke coverage validates:
  - `/v1/health`
  - `/v1/usage`
  - `/v1/search`
  - `/v1/fetch`
  - `/v1/research/jobs` + `/v1/jobs/:id`
  - MCP initialize over the HTTP JSON-RPC endpoint, if a public MCP URL is configured

## Remaining External Rollout Work

- Deploy the current code to the public host. This repository change does not deploy by itself.
- Configure `FETCHIUM_DEPLOY_HOST`, `FETCHIUM_DEPLOY_USER`, `FETCHIUM_DEPLOY_SSH_KEY`, and `FETCHIUM_DEPLOY_PATH` for the deploy workflow.
- Configure Jenkins credentials documented in `docs/jenkins-cicd-server100.md` if Jenkins on `server100` is the primary CD path.
- Configure GitHub Actions secrets and vars for public smoke and package publishing.
- Deploy the HTTP MCP endpoint and set `PUBLIC_FETCHIUM_MCP_URL` to the public `/mcp` URL.

## Release Checklist

```bash
cargo fmt
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
python -m unittest adapters.langchain.tests.test_retriever adapters.crewai.tests.test_tool
npm pack --dry-run ./packages/npm
```

## Build Environment Prerequisites

- `cargo clippy --workspace --all-targets --all-features -- -D warnings` requires native build tooling for enabled optional dependencies. On Linux that means standard libc headers plus the usual C/C++ toolchain for crates such as `chromiumoxide`, `fastembed`, or other native transitive dependencies.
