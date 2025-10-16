pub async fn handler() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({"status": "ok"}))
}
