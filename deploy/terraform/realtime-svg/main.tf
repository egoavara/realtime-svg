locals {
  app_name = "realtime-svg"
  labels = merge(
    {
      "app.kubernetes.io/name"       = local.app_name
      "app.kubernetes.io/instance"   = local.app_name
      "app.kubernetes.io/version"    = var.image_tag
      "app.kubernetes.io/managed-by" = "terraform"
    },
    var.labels
  )

  redis_url = var.redis_enabled ? (
    var.redis_password != "" ?
    "redis://:${var.redis_password}@${local.app_name}-redis:6379/" :
    "redis://${local.app_name}-redis:6379/"
    ) : (
    var.redis_external_url != "" ?
    var.redis_external_url :
    ""
  )
}
