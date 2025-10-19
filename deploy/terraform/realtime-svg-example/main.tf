terraform {
  required_version = ">= 1.0"

  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
  }
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = "{{ kube_context 이름 }}"
}

module "realtime_svg" {
  source = "../realtime-svg"

  namespace = "default"
  replicas  = 2

  image_tag = "v0.1.4"

  redis_enabled = true

  ingress_enabled = true
  ingress_host    = "realtime-svg.egoavara.net"

  service_type = "ClusterIP"
  service_port = 80
}

output "service_name" {
  value = module.realtime_svg.service_name
}

output "ingress_url" {
  value = module.realtime_svg.ingress_url
}
