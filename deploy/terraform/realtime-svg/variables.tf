variable "namespace" {
  description = "Kubernetes namespace for deployment"
  type        = string
  default     = "default"
}

variable "image_repository" {
  description = "Container image repository"
  type        = string
  default     = "ghcr.io/egoavara/realtime-svg"
}

# AUTOUPDATE: prepare-release workflow updates this version - do not edit manually
variable "image_tag" {
  description = "Container image tag"
  type        = string
  default     = "v0.1.10"
}

variable "image_pull_policy" {
  description = "Image pull policy (IfNotPresent, Always, Never)"
  type        = string
  default     = "IfNotPresent"

  validation {
    condition     = contains(["IfNotPresent", "Always", "Never"], var.image_pull_policy)
    error_message = "image_pull_policy must be one of: IfNotPresent, Always, Never"
  }
}

variable "replicas" {
  description = "Number of replicas"
  type        = number
  default     = 2

  validation {
    condition     = var.replicas > 0
    error_message = "replicas must be greater than 0"
  }
}

variable "resources_requests_cpu" {
  description = "CPU request (e.g., 100m, 1)"
  type        = string
  default     = "100m"
}

variable "resources_requests_memory" {
  description = "Memory request (e.g., 128Mi, 1Gi)"
  type        = string
  default     = "128Mi"
}

variable "resources_limits_cpu" {
  description = "CPU limit (e.g., 500m, 2)"
  type        = string
  default     = "500m"
}

variable "resources_limits_memory" {
  description = "Memory limit (e.g., 512Mi, 2Gi)"
  type        = string
  default     = "512Mi"
}

variable "redis_enabled" {
  description = "Enable in-cluster Redis deployment"
  type        = bool
  default     = true
}

variable "redis_image_repository" {
  description = "Redis image repository (only used if redis_enabled=true)"
  type        = string
  default     = "redis"
}

variable "redis_image_tag" {
  description = "Redis image tag (only used if redis_enabled=true)"
  type        = string
  default     = "8-alpine"
}

variable "redis_password" {
  description = "Redis password (optional, for in-cluster Redis)"
  type        = string
  default     = ""
  sensitive   = true
}

variable "redis_resources_requests_cpu" {
  description = "Redis CPU request (only used if redis_enabled=true)"
  type        = string
  default     = "100m"
}

variable "redis_resources_requests_memory" {
  description = "Redis memory request (only used if redis_enabled=true)"
  type        = string
  default     = "128Mi"
}

variable "redis_resources_limits_cpu" {
  description = "Redis CPU limit (only used if redis_enabled=true)"
  type        = string
  default     = "200m"
}

variable "redis_resources_limits_memory" {
  description = "Redis memory limit (only used if redis_enabled=true)"
  type        = string
  default     = "256Mi"
}

variable "redis_external_url" {
  description = "External Redis URL (required if redis_enabled=false, e.g., redis://host:6379/)"
  type        = string
  default     = ""
  sensitive   = true
}

variable "ingress_enabled" {
  description = "Enable Ingress resource creation"
  type        = bool
  default     = true
}

variable "ingress_host" {
  description = "Ingress hostname"
  type        = string
  default     = "realtime-svg.example.com"
}

variable "ingress_path" {
  description = "Ingress path"
  type        = string
  default     = "/"
}

variable "ingress_path_type" {
  description = "Ingress path type (Prefix, Exact)"
  type        = string
  default     = "Prefix"

  validation {
    condition     = contains(["Prefix", "Exact"], var.ingress_path_type)
    error_message = "ingress_path_type must be one of: Prefix, Exact"
  }
}

variable "ingress_annotations" {
  description = "Custom annotations for Ingress"
  type        = map(string)
  default     = {}
}

variable "ingress_tls_enabled" {
  description = "Enable TLS for Ingress"
  type        = bool
  default     = false
}

variable "ingress_tls_secret_name" {
  description = "TLS secret name (required if ingress_tls_enabled=true)"
  type        = string
  default     = ""
}

variable "service_type" {
  description = "Service type (ClusterIP, NodePort, LoadBalancer)"
  type        = string
  default     = "ClusterIP"

  validation {
    condition     = contains(["ClusterIP", "NodePort", "LoadBalancer"], var.service_type)
    error_message = "service_type must be one of: ClusterIP, NodePort, LoadBalancer"
  }
}

variable "service_port" {
  description = "Service port"
  type        = number
  default     = 80

  validation {
    condition     = var.service_port > 0 && var.service_port <= 65535
    error_message = "service_port must be between 1 and 65535"
  }
}

variable "config_log_level" {
  description = "Log level (info, debug, warn, error)"
  type        = string
  default     = "info"

  validation {
    condition     = contains(["info", "debug", "warn", "error"], var.config_log_level)
    error_message = "config_log_level must be one of: info, debug, warn, error"
  }
}

variable "config_port" {
  description = "Application container port"
  type        = number
  default     = 8080

  validation {
    condition     = var.config_port > 0 && var.config_port <= 65535
    error_message = "config_port must be between 1 and 65535"
  }
}

variable "secret_api_key" {
  description = "API key (optional, will be base64 encoded)"
  type        = string
  default     = ""
  sensitive   = true
}

variable "labels" {
  description = "Additional labels to apply to all resources"
  type        = map(string)
  default     = {}
}

variable "common_annotations" {
  description = "Common annotations to apply to all resources"
  type        = map(string)
  default     = {}
}

variable "deployment_annotations" {
  description = "Annotations for Deployment resource"
  type        = map(string)
  default     = {}
}

variable "service_annotations" {
  description = "Annotations for Service resource"
  type        = map(string)
  default     = {}
}

variable "configmap_annotations" {
  description = "Annotations for ConfigMap resource"
  type        = map(string)
  default     = {}
}

variable "secret_annotations" {
  description = "Annotations for Secret resource"
  type        = map(string)
  default     = {}
}

variable "redis_deployment_annotations" {
  description = "Annotations for Redis Deployment resource (only used if redis_enabled=true)"
  type        = map(string)
  default     = {}
}

variable "redis_service_annotations" {
  description = "Annotations for Redis Service resource (only used if redis_enabled=true)"
  type        = map(string)
  default     = {}
}
