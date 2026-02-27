// AI Inference Service - placeholder implementation

use axum::{
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Starting AI Inference Service...");

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3003";
    tracing::info!("AI Inference service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "ai-inference"
    }))
}
