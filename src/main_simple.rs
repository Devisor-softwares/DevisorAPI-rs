use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct SimpleState {
    pub message: String,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(SimpleState {
        message: "Game Server Panel API - Simple Mode".to_string(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/v1/status", get(api_status))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("📖 Available endpoints:");
    println!("  GET / - API root");
    println!("  GET /health - Health check");
    println!("  GET /api/v1/status - API status");

    axum::serve(listener, app).await.unwrap();
}

async fn root(State(state): State<Arc<SimpleState>>) -> Json<Value> {
    Json(json!({
        "message": state.message,
        "version": "0.1.0",
        "status": "running"
    }))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "game-server-panel"
    }))
}

async fn api_status(State(state): State<Arc<SimpleState>>) -> Json<Value> {
    Json(json!({
        "api": "Game Server Panel Backend",
        "version": "0.1.0",
        "mode": "simple (no database)",
        "features": [
            "HTTP server",
            "CORS support",
            "JSON responses",
            "Health checks"
        ],
        "database": {
            "status": "not connected",
            "note": "Run with PostgreSQL for full functionality"
        },
        "endpoints": {
            "authentication": "disabled (requires database)",
            "server_management": "disabled (requires database)",
            "monitoring": "disabled (requires database)"
        }
    }))
}
