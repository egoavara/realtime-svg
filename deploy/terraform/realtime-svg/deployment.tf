resource "kubernetes_deployment" "realtime_svg" {
  metadata {
    name      = local.app_name
    namespace = var.namespace
    labels    = local.labels
  }

  spec {
    replicas = var.replicas

    strategy {
      type = "RollingUpdate"
      rolling_update {
        max_unavailable = 1
        max_surge       = 1
      }
    }

    selector {
      match_labels = {
        "app.kubernetes.io/name"     = local.app_name
        "app.kubernetes.io/instance" = local.app_name
      }
    }

    template {
      metadata {
        labels = local.labels
        annotations = {
          "checksum/config" = sha256(jsonencode(kubernetes_config_map.realtime_svg.data))
          "checksum/secret" = sha256(jsonencode(kubernetes_secret.realtime_svg.data))
        }
      }

      spec {
        container {
          name              = local.app_name
          image             = "${var.image_repository}:${var.image_tag}"
          image_pull_policy = var.image_pull_policy

          port {
            name           = "http"
            container_port = var.config_port
            protocol       = "TCP"
          }

          env {
            name  = "HOST"
            value = "0.0.0.0"
          }

          env {
            name = "REDIS_URL"
            value_from {
              config_map_key_ref {
                name = kubernetes_config_map.realtime_svg.metadata[0].name
                key  = "REDIS_URL"
              }
            }
          }

          env {
            name = "LOG_LEVEL"
            value_from {
              config_map_key_ref {
                name = kubernetes_config_map.realtime_svg.metadata[0].name
                key  = "LOG_LEVEL"
              }
            }
          }

          env {
            name = "PORT"
            value_from {
              config_map_key_ref {
                name = kubernetes_config_map.realtime_svg.metadata[0].name
                key  = "PORT"
              }
            }
          }

          dynamic "env" {
            for_each = var.secret_api_key != "" ? [1] : []
            content {
              name = "API_KEY"
              value_from {
                secret_key_ref {
                  name = kubernetes_secret.realtime_svg.metadata[0].name
                  key  = "apiKey"
                }
              }
            }
          }

          resources {
            requests = {
              cpu    = var.resources_requests_cpu
              memory = var.resources_requests_memory
            }
            limits = {
              cpu    = var.resources_limits_cpu
              memory = var.resources_limits_memory
            }
          }

          liveness_probe {
            http_get {
              path = "/health"
              port = var.config_port
            }
            initial_delay_seconds = 10
            period_seconds        = 10
            timeout_seconds       = 5
            failure_threshold     = 3
          }

          readiness_probe {
            http_get {
              path = "/ready"
              port = var.config_port
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

resource "kubernetes_service" "realtime_svg" {
  metadata {
    name      = local.app_name
    namespace = var.namespace
    labels    = local.labels
  }

  spec {
    type = var.service_type

    selector = {
      "app.kubernetes.io/name"     = local.app_name
      "app.kubernetes.io/instance" = local.app_name
    }

    port {
      name        = "http"
      port        = var.service_port
      target_port = "http"
      protocol    = "TCP"
    }
  }
}
