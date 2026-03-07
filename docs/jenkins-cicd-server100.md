# Jenkins CI/CD on server100

This repository is set up to use Jenkins on `server100` to deploy Fetchium onto `server15`.

## Target topology

- Jenkins runner: `server100`
- Deployment target: `server15`
- Public domains:
  - `***REMOVED***`
  - `***REMOVED***/mcp`

## Files used

- Pipeline: [Jenkinsfile](/home/echo/projects/Fetchium/Jenkinsfile)
- Production env: [fetchium.env.production](/home/echo/projects/Fetchium/infra/fetchium.env.production)
- Production compose: [docker-compose.prod.yml](/home/echo/projects/Fetchium/infra/docker-compose.prod.yml)
- Deploy script: [deploy.sh](/home/echo/projects/Fetchium/scripts/deploy.sh)
- Traefik route snippet: [traefik.fetchium.yml](/home/echo/projects/Fetchium/infra/traefik.fetchium.yml)

## Jenkins credentials to create

- `fetchium-deploy-ssh`
  - SSH private key for logging into `server15`
- `fetchium-deploy-host`
  - Secret text: `103.204.87.226`
- `fetchium-deploy-user`
  - Secret text: `echo`
- `fetchium-deploy-path`
  - Secret text: `/home/echo/projects/Fetchium`

## Server15 preparation

1. Copy [fetchium.env.production](/home/echo/projects/Fetchium/infra/fetchium.env.production) to the target host and replace placeholders.
2. Ensure Docker Engine and Docker Compose v2 are installed on `server15`.
3. Install the Traefik snippet from [traefik.fetchium.yml](/home/echo/projects/Fetchium/infra/traefik.fetchium.yml) into the dynamic config on `server15`.
4. Reload Traefik.

## Manual first deploy

```bash
cd /home/echo/projects/Fetchium
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

## Health checks

```bash
curl ***REMOVED***/v1/health
curl -X POST ***REMOVED***/mcp \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
```
