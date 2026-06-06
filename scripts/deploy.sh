#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${FETCHIUM_COMPOSE_FILE:-infra/docker-compose.prod.yml}"
PROJECT_DIR="${FETCHIUM_DEPLOY_DIR:-$ROOT_DIR}"
ENV_FILE="${FETCHIUM_ENV_FILE:-infra/fetchium.env.production}"

cd "$PROJECT_DIR"

if [[ ! -f "$COMPOSE_FILE" ]]; then
  echo "Compose file not found: $COMPOSE_FILE" >&2
  exit 1
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Env file not found: $ENV_FILE" >&2
  exit 1
fi

docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" pull --ignore-pull-failures
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --build
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" ps
