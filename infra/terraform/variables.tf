variable "app_prefix" {
  description = "Prefix for all Fly app names — must be globally unique across Fly.io"
  type        = string
  default     = "rotor"
}

variable "region" {
  description = "Fly.io region to deploy to"
  type        = string
  default     = "iad"
}

variable "iggy_username" {
  description = "Iggy root username"
  type        = string
  default     = "iggy"
}

variable "iggy_password" {
  description = "Iggy root password"
  type        = string
  sensitive   = true
}

variable "cloudflare_account_id" {
  description = "Cloudflare account ID (find in the Cloudflare dashboard URL)"
  type        = string
}
