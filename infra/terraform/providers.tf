terraform {
  required_version = ">= 1.6"

  required_providers {
    fly = {
      source  = "fly-apps/fly"
      version = "~> 0.0.23"
    }
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.0"
    }
  }
}

provider "fly" {
  # Auth: export FLY_API_TOKEN=$(fly auth token)
}

provider "cloudflare" {
  # Auth: export CLOUDFLARE_API_TOKEN=<your token>
}
