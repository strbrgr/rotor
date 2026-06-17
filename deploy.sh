#!/usr/bin/env bash
# Usage: ./deploy.sh [app-prefix]
# Requires: terraform, flyctl, docker, wrangler
# Env vars: FLY_API_TOKEN, CLOUDFLARE_API_TOKEN, TF_VAR_iggy_password, TF_VAR_cloudflare_account_id
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TF="$REPO/infra/terraform"
PREFIX="${1:-rotor}"
REGION="${TF_VAR_region:-iad}"
IGGY_PASS="${TF_VAR_iggy_password:?TF_VAR_iggy_password must be set}"

step() { echo ""; echo "[deploy] ── $* ──"; }

# Returns true if a machine named $2 already exists in app $1
machine_exists() {
  fly machine list --app "$1" --json 2>/dev/null | grep -q "\"name\":\"$2\""
}

# ── 1. Terraform: apps, IPs, Cloudflare Pages project ─────────────────────────
step "Terraform: provisioning apps, IPs, and Cloudflare Pages"
cd "$TF"
terraform init -input=false
terraform apply \
  -var="app_prefix=$PREFIX" \
  -input=false \
  -auto-approve
cd "$REPO"

# ── 2. Iggy volume (flyctl — Fly deprecated volume ops in GraphQL API) ─────────
step "flyctl: creating Iggy volume (skipped if exists)"
fly volumes list --app "${PREFIX}-iggy" --json 2>/dev/null \
  | grep -q '"name":"iggy_data"' \
  || fly volumes create iggy_data \
       --app "${PREFIX}-iggy" \
       --region "$REGION" \
       --size 1

# ── 3. Build and push Docker images ───────────────────────────────────────────
step "Docker: authenticating with Fly registry"
fly auth token | docker login registry.fly.io --username x --password-stdin

for svc in sse gateway sensor; do
  step "Docker: building $svc"
  docker build \
    --platform linux/amd64 \
    --target "$svc" \
    -t "registry.fly.io/${PREFIX}-${svc}:latest" \
    -f "$REPO/docker/Dockerfile" \
    "$REPO"

  step "Docker: pushing $svc"
  docker push "registry.fly.io/${PREFIX}-${svc}:latest"
done

# ── 4. Deploy machines via flyctl ─────────────────────────────────────────────
IGGY_ENV=(
  --env "IGGY_ROOT_USERNAME=iggy"
  --env "IGGY_ROOT_PASSWORD=$IGGY_PASS"
  --env "IGGY_STREAM_NAME=sample-stream"
  --env "IGGY_TOPIC_NAME=sample-topic"
  --env "IGGY_PARTITION_ID=0"
  --env "IGGY_SERVER_ADDRESS=${PREFIX}-iggy.internal:8090"
)

step "flyctl: deploying Iggy"
machine_exists "${PREFIX}-iggy" "iggy" \
  || fly machine run apache/iggy:latest \
       --app "${PREFIX}-iggy" --name iggy --region "$REGION" \
       --env "IGGY_TCP_ADDRESS=0.0.0.0:8090" \
       --env "IGGY_ROOT_USERNAME=iggy" \
       --env "IGGY_ROOT_PASSWORD=$IGGY_PASS" \
       --volume "iggy_data:/local_data"

step "flyctl: waiting for Iggy to start"
sleep 10

step "flyctl: deploying gateway"
machine_exists "${PREFIX}-gateway" "gateway" \
  || fly machine run "registry.fly.io/${PREFIX}-gateway:latest" \
       --app "${PREFIX}-gateway" --name gateway --region "$REGION" \
       "${IGGY_ENV[@]}"

step "flyctl: deploying SSE server"
machine_exists "${PREFIX}-sse" "sse" \
  || fly machine run "registry.fly.io/${PREFIX}-sse:latest" \
       --app "${PREFIX}-sse" --name sse --region "$REGION" \
       "${IGGY_ENV[@]}" \
       --port "443:3001/tcp:tls:http" \
       --port "80:3001/tcp:http"

step "flyctl: deploying sensors"
machine_exists "${PREFIX}-sensor" "sensor-100ms" \
  || fly machine run "registry.fly.io/${PREFIX}-sensor:latest" sensor 100 \
       --app "${PREFIX}-sensor" --name sensor-100ms --region "$REGION" \
       --env "GATEWAY_ADDRESS=${PREFIX}-gateway.internal:8080"

machine_exists "${PREFIX}-sensor" "sensor-150ms" \
  || fly machine run "registry.fly.io/${PREFIX}-sensor:latest" sensor 150 \
       --app "${PREFIX}-sensor" --name sensor-150ms --region "$REGION" \
       --env "GATEWAY_ADDRESS=${PREFIX}-gateway.internal:8080"

# ── 5. Build UI and deploy to Cloudflare Pages ────────────────────────────────
SSE_URL="https://${PREFIX}-sse.fly.dev"
step "UI: building (VITE_SSE_URL=$SSE_URL/events)"
cd "$REPO/ui"
npm install --silent
VITE_SSE_URL="$SSE_URL/events" npm run build

step "UI: deploying to Cloudflare Pages"
npx wrangler pages deploy dist --project-name="${PREFIX}-ui"
cd "$REPO"

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "[deploy] Done!"
echo "  UI:  https://${PREFIX}-ui.pages.dev"
echo "  SSE: $SSE_URL"
