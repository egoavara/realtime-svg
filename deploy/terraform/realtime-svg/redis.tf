resource "kubernetes_deployment" "redis" {
  count = var.redis_enabled ? 1 : 0

  metadata {
    name        = "${local.app_name}-redis"
    namespace   = var.namespace
    labels      = local.redis_labels
    annotations = merge(var.common_annotations, var.redis_deployment_annotations)
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        "app.kubernetes.io/name"      = local.app_name
        "app.kubernetes.io/instance"  = local.app_name
        "app.kubernetes.io/component" = "redis"
      }
    }

    template {
      metadata {
        labels = local.redis_labels
      }

      spec {
        container {
          name              = "redis"
          image             = "${var.redis_image_repository}:${var.redis_image_tag}"
          image_pull_policy = "IfNotPresent"

          port {
            name           = "redis"
            container_port = 6379
            protocol       = "TCP"
          }

          args = var.redis_password != "" ? [
            "--requirepass",
            var.redis_password,
            "--save",
            "",
            "--appendonly",
            "no",
            "--maxmemory",
            "256mb",
            "--maxmemory-policy",
            "allkeys-lru"
            ] : [
            "--save",
            "",
            "--appendonly",
            "no",
            "--maxmemory",
            "256mb",
            "--maxmemory-policy",
            "allkeys-lru"
          ]

          resources {
            requests = {
              cpu    = var.redis_resources_requests_cpu
              memory = var.redis_resources_requests_memory
            }
            limits = {
              cpu    = var.redis_resources_limits_cpu
              memory = var.redis_resources_limits_memory
            }
          }

          liveness_probe {
            exec {
              command = var.redis_password != "" ? [
                "redis-cli",
                "-a",
                var.redis_password,
                "ping"
                ] : [
                "redis-cli",
                "ping"
              ]
            }
            initial_delay_seconds = 5
            period_seconds        = 10
            timeout_seconds       = 5
            failure_threshold     = 3
          }

          readiness_probe {
            exec {
              command = var.redis_password != "" ? [
                "redis-cli",
                "-a",
                var.redis_password,
                "ping"
                ] : [
                "redis-cli",
                "ping"
              ]
            }
            initial_delay_seconds = 5
            period_seconds        = 5
            timeout_seconds       = 3
            failure_threshold     = 3
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "redis" {
  count = var.redis_enabled ? 1 : 0

  metadata {
    name        = "${local.app_name}-redis"
    namespace   = var.namespace
    labels      = local.redis_labels
    annotations = merge(var.common_annotations, var.redis_service_annotations)
  }

  spec {
    type = "ClusterIP"

    selector = {
      "app.kubernetes.io/name"      = local.app_name
      "app.kubernetes.io/instance"  = local.app_name
      "app.kubernetes.io/component" = "redis"
    }

    port {
      name        = "redis"
      port        = 6379
      target_port = "redis"
      protocol    = "TCP"
    }
  }
}
