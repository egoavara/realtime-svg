output "namespace" {
  description = "Kubernetes namespace where resources are deployed"
  value       = var.namespace
}

output "deployment_name" {
  description = "Name of the Deployment resource"
  value       = kubernetes_deployment.realtime_svg.metadata[0].name
}

output "service_name" {
  description = "Name of the Service resource"
  value       = kubernetes_service.realtime_svg.metadata[0].name
}

output "service_type" {
  description = "Type of the Service (ClusterIP, NodePort, LoadBalancer)"
  value       = kubernetes_service.realtime_svg.spec[0].type
}

output "service_port" {
  description = "Service port"
  value       = kubernetes_service.realtime_svg.spec[0].port[0].port
}

output "ingress_hostname" {
  description = "Ingress hostname (if enabled)"
  value       = var.ingress_enabled ? var.ingress_host : null
}

output "ingress_url" {
  description = "Full URL to access the application via Ingress (if enabled)"
  value       = var.ingress_enabled ? (var.ingress_tls_enabled ? "https://${var.ingress_host}" : "http://${var.ingress_host}") : null
}

output "redis_service_name" {
  description = "Name of the Redis Service (if redis_enabled=true)"
  value       = var.redis_enabled ? kubernetes_service.redis[0].metadata[0].name : null
}

output "redis_endpoint" {
  description = "Redis connection endpoint"
  value = var.redis_enabled ? (
    var.redis_password != "" ?
    "redis://:***@${kubernetes_service.redis[0].metadata[0].name}:6379/" :
    "redis://${kubernetes_service.redis[0].metadata[0].name}:6379/"
    ) : (
    var.redis_external_url != "" ? "redis://***" : null
  )
  sensitive = true
}

output "configmap_name" {
  description = "Name of the ConfigMap resource"
  value       = kubernetes_config_map.realtime_svg.metadata[0].name
}

output "secret_name" {
  description = "Name of the Secret resource"
  value       = kubernetes_secret.realtime_svg.metadata[0].name
}
