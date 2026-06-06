# Contributing to Fetchium

First off — thank you for considering contributing to Fetchium! 🦀

Fetchium is an open-source, Rust-native retrieval layer that lets humans and AI agents find,
fetch, and verify information from the open web. It's built in the open, and contributions of all
sizes are welcome — from fixing a typo to adding a new search backend or fetch mode.

This document explains how to get set up, what we expect, and how to get your change merged.

---

## Table of contents
- [Code of Conduct](#code-of-conduct)
- [Ways to contribute](#ways-to-contribute)
- [Project layout](#project-layout)
- [Development setup](#development-setup)
- [Building and testing](#building-and-testing)
- [Coding standards](#coding-standards)
- [Commit and PR guidelines](#commit-and-pr-guidelines)
- [Reporting bugs](#reporting-bugs)
- [Proposing features](#proposing-features)
- [Security issues](#security-issues)
- [License](#license)

---

## Code of Conduct

This project and everyone participating in it is governed by the
[Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold it.
Please report unacceptable behavior via the channels listed there.

## Ways to contribute

You don't have to write Rust to help:

- 🐛 **Report bugs** — open a [bug report](https://github.com/zuhabul/Fetchium/issues/new/choose).
- 💡 **Suggest features** — open a feature request describing the use case.
- 📖 **Improve docs** — fix typos, clarify guides in `docs/`, or add examples.
- 🔌 **Add a backend or fetch mode** — a new search provider, extractor, or output format.
- 🧪 **Write tests** — coverage for edge cases is always valuable.
- 🤖 **Improve agent integration** — the MCP server (`fetchium-mcp`) and LangChain/CrewAI adapters.

Looking for a place to start? Check issues labeled
[`good first issue`](https://github.com/zuhabul/Fetchium/labels/good%20first%20issue) and
[`help wanted`](https://github.com/zuhabul/Fetchium/labels/help%20wanted).

## Project layout

Fetchium is a Cargo workspace:

| Crate | Path | Responsibility |
|-------|------|----------------|
| `fetchium-core` | `crates/fetchium-core` | Search, extract, rank, validate, cache — the retrieval engine |
| `fetchium-cli`  | `crates/fetchium-cli`  | The `fetchium` command-line binary |
| `fetchium-mcp`  | `crates/fetchium-mcp`  | Model Context Protocol server for AI agents |
| `fetchium-api`  | `crates/fetchium-api`  | REST API server |

User-facing documentation lives in `docs/` (guides + architecture).

## Development setup

You'll need:

- **Rust 1.75+** (install via [rustup](https://rustup.rs))
- A C toolchain (for some native dependencies)

```bash
git clone https://github.com/zuhabul/Fetchium
cd Fetchium
cargo build
```

Optional runtime tools (only needed for specific features — run `fetchium doctor` to check):
Ollama (local AI), Chromium (JS-rendered pages), Pandoc/Typst (report generation), Tesseract (OCR).

## Building and testing

Before opening a PR, make sure these pass locally — they are exactly what CI runs:

```bash
# Format
cargo fmt --all

# Lint (CI denies warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Tests (default features)
cargo test --workspace --lib --bins --tests

# Doc tests
cargo test --doc --workspace
```

> **Note on features:** default features build with no exotic native toolchains. Heavy optional
> features (e.g. `ort`/ONNX, local llama inference) require platform-specific native libraries and
> are intentionally **not** part of the default CI matrix. Build them explicitly if you work on them.

## Coding standards

- **Format with `rustfmt`** and keep `clippy` clean (`-D warnings`).
- **Small, focused changes.** One logical change per PR.
- **Errors are explicit** — return `Result`, don't `unwrap()` in library code paths.
- **Add tests** for new behavior. Network-dependent tests should be gated/mocked so the default
  test run stays deterministic and offline.
- **Document public items** with `///` doc comments; keep `cargo doc` warning-free.

## Commit and PR guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Brave search backend
fix(cli): correct exit code on empty query
docs: clarify MCP setup
chore: bump dependencies
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`.

To open a PR:
1. Fork the repo and create a branch (`feat/my-thing`).
2. Make your change with tests + docs.
3. Run the full check list above.
4. Open a PR against `main` describing **what** and **why**. Link any related issue.
5. A maintainer will review. CI must be green before merge.

## Reporting bugs

Open an issue with: what you did, what you expected, what happened, your OS + `fetchium --version`,
and any relevant logs (run with `--verbose`). A minimal reproduction is gold.

## Proposing features

Open a feature request describing the **use case** first, then the proposed solution. For larger
changes, it's best to discuss in an issue before writing code so we can agree on the approach.

## Security issues

**Do not open public issues for security vulnerabilities.** See [SECURITY.md](SECURITY.md) for how
to report privately.

## License

By contributing, you agree that your contributions will be dual-licensed under the
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE) licenses, the same terms as the project.
