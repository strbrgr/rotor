output "app_prefix" {
  value = var.app_prefix
}

output "sse_url" {
  description = "Public URL of the SSE server"
  value       = "https://${fly_app.sse.name}.fly.dev"
}

output "ui_url" {
  description = "Cloudflare Pages URL for the UI"
  value       = "https://${cloudflare_pages_project.ui.name}.pages.dev"
}
