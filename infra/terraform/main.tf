# ── Apps ──────────────────────────────────────────────────────────────────────

resource "fly_app" "iggy"    { name = "${var.app_prefix}-iggy" }
resource "fly_app" "gateway" { name = "${var.app_prefix}-gateway" }
resource "fly_app" "sse"     { name = "${var.app_prefix}-sse" }
resource "fly_app" "sensor"  { name = "${var.app_prefix}-sensor" }

# ── Public IPs (SSE is the only internet-facing service) ──────────────────────

resource "fly_ip" "sse_v4" {
  app  = fly_app.sse.name
  type = "v4"
}

resource "fly_ip" "sse_v6" {
  app  = fly_app.sse.name
  type = "v6"
}

# ── UI — static Svelte build on Cloudflare Pages ──────────────────────────────

resource "cloudflare_pages_project" "ui" {
  account_id        = var.cloudflare_account_id
  name              = "${var.app_prefix}-ui"
  production_branch = "main"

  build_config {
    build_command   = "cd ui && npm install && npm run build"
    destination_dir = "ui/dist"
  }

  deployment_configs {
    production {
      environment_variables = {
        VITE_SSE_URL = "https://${fly_app.sse.name}.fly.dev/events"
      }
    }
  }
}
