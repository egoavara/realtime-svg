resource "kubernetes_ingress_v1" "realtime_svg" {
  count = var.ingress_enabled ? 1 : 0

  metadata {
    name        = local.app_name
    namespace   = var.namespace
    labels      = local.labels
    annotations = var.ingress_annotations
  }

  spec {
    dynamic "tls" {
      for_each = var.ingress_tls_enabled ? [1] : []
      content {
        hosts = [var.ingress_host]
        secret_name = var.ingress_tls_secret_name != "" ? var.ingress_tls_secret_name : (
          var.ingress_tls_enabled ?
          var.ingress_tls_secret_name :
          null
        )
      }
    }

    rule {
      host = var.ingress_host

      http {
        path {
          path      = var.ingress_path
          path_type = var.ingress_path_type

          backend {
            service {
              name = kubernetes_service.realtime_svg.metadata[0].name
              port {
                number = var.service_port
              }
            }
          }
        }
      }
    }
  }
}
