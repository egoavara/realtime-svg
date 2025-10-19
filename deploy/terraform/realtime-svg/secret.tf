resource "kubernetes_secret" "realtime_svg" {
  metadata {
    name      = "${local.app_name}-secret"
    namespace = var.namespace
    labels    = local.labels
  }

  type = "Opaque"

  data = merge(
    var.secret_api_key != "" ? {
      apiKey = base64encode(var.secret_api_key)
    } : {}
  )
}
