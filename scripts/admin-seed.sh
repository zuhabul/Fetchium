#!/bin/bash
# Fetchium admin seed data — creates realistic test data in admin.db
# Usage: ./scripts/admin-seed.sh
set -euo pipefail

API="http://localhost:3050/internal/admin"
BOOTSTRAP_SECRET="${***REMOVED***:-}"

if [ -z "$BOOTSTRAP_SECRET" ]; then
  echo "Error: ***REMOVED*** env var required"
  exit 1
fi

echo "Seeding Fetchium admin data..."

# 1. Create staff accounts (if not exists)
echo "Creating staff accounts..."
for role in ops support finance growth readonly; do
  curl -sf -X POST "$API/auth/bootstrap" \
    -H "Content-Type: application/json" \
    -H "X-Bootstrap-Secret: $BOOTSTRAP_SECRET" \
    -d "{\"name\":\"${role^} User\",\"email\":\"${role}@fetchium.com\",\"password\":\"Fetchium2026!\"}" \
    2>/dev/null || true
done

# 2. Create sample organizations
echo "Creating sample organizations..."
# Get owner session first
SESSION=$(curl -sf -X POST "$API/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@fetchium.com","password":"KtZbedf"}' | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('token',''))" 2>/dev/null || echo "")

if [ -z "$SESSION" ]; then
  echo "Warning: Could not get admin session — skipping org/key seeding"
  echo "Seed script completed (partial)"
  exit 0
fi

AUTH="-H \"Authorization: Bearer $SESSION\""

PLANS=("free" "starter" "pro" "enterprise")
STAGES=("prospect" "trial" "customer" "expansion" "churned")

for i in $(seq 1 20); do
  PLAN=${PLANS[$((RANDOM % 4))]}
  NAME="Org $i"
  SLUG="org-$i"
  curl -sf -X POST "$API/orgs" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $SESSION" \
    -d "{\"name\":\"$NAME\",\"slug\":\"$SLUG\",\"plan\":\"$PLAN\",\"owner_email\":\"owner${i}@org${i}.com\"}" \
    2>/dev/null || true
done

# 3. Create sample incidents
echo "Creating sample incidents..."
for severity in low medium high critical; do
  curl -sf -X POST "$API/incidents" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $SESSION" \
    -d "{\"title\":\"Sample $severity incident\",\"severity\":\"$severity\"}" \
    2>/dev/null || true
done

# 4. Create sample feature flags
echo "Creating feature flags..."
for flag in "search.enabled" "research.enabled" "proxy.residential_enabled" "rate_limit.strict" "new_signups.enabled" "beta.headless_extraction"; do
  curl -sf -X POST "$API/flags" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $SESSION" \
    -d "{\"key\":\"$flag\",\"description\":\"Kill switch: $flag\",\"enabled_globally\":true,\"is_dangerous\":false}" \
    2>/dev/null || true
done

# 5. Create sample support tickets
echo "Creating support tickets..."
for priority in normal normal high urgent normal low; do
  curl -sf -X POST "$API/support/tickets" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $SESSION" \
    -d "{\"subject\":\"Customer inquiry ($priority priority)\",\"priority\":\"$priority\",\"status\":\"open\"}" \
    2>/dev/null || true
done

echo ""
echo "Seed complete!"
echo "  Staff accounts: ops, support, finance, growth, readonly @fetchium.com (pw: Fetchium2026!)"
echo "  Sample orgs, incidents, flags, and tickets created"
