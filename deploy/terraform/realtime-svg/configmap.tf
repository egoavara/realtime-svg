resource "kubernetes_config_map" "realtime_svg" {
  metadata {
    name        = "${local.app_name}-config"
    namespace   = var.namespace
    labels      = local.labels
    annotations = merge(var.common_annotations, var.configmap_annotations)
  }

  data = {
    REDIS_URL = local.redis_url
    LOG_LEVEL = var.config_log_level
    PORT      = tostring(var.config_port)
  }
}
