#!/usr/bin/env sh
# Setup local development environment.
# Run once after cloning: sh scripts/setup-dev.sh
set -eu

echo "Setting up Fetchium development environment..."

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
HOOKS_DIR="${REPO_ROOT}/.git/hooks"

# Install commit-msg hook for conventional commits
cp "${REPO_ROOT}/scripts/hooks/commit-msg" "${HOOKS_DIR}/commit-msg"
chmod +x "${HOOKS_DIR}/commit-msg"
echo "✓ commit-msg hook installed"

# Install pre-commit hook
cp "${REPO_ROOT}/scripts/hooks/pre-commit" "${HOOKS_DIR}/pre-commit"
chmod +x "${HOOKS_DIR}/pre-commit"
echo "✓ pre-commit hook installed"

echo ""
echo "Development environment ready."
echo ""
echo "Commit format: <type>(<scope>): <description>"
echo ""
echo "Types and their version impact:"
echo "  feat:      → minor bump  (1.0.0 → 1.1.0)"
echo "  fix:       → patch bump  (1.0.0 → 1.0.1)"
echo "  feat!:     → major bump  (1.0.0 → 2.0.0)  ← BREAKING CHANGE"
echo "  perf:      → patch bump"
echo "  docs:      → no version bump"
echo "  chore:     → no version bump"
echo "  refactor:  → no version bump"
echo "  test:      → no version bump"
echo "  ci:        → no version bump"
echo ""
echo "Examples:"
echo "  git commit -m 'feat: add cross-lingual query expansion'"
echo "  git commit -m 'fix: handle rate limit retry correctly'"
echo "  git commit -m 'feat(rank)!: redesign HyperFusion signal weights'"
echo ""
echo "Publishing is fully AUTOMATED:"
echo "  1. Commit with correct type → push to main"
echo "  2. release-please opens/updates a Release PR"
echo "  3. Merge the Release PR → binaries built → npm published → Homebrew updated"
echo ""
echo "Never manually edit Cargo.toml version or create git tags."
