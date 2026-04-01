mod auth;
mod database;
mod handlers;
mod logging;
mod middleware;
mod models;
mod services;
mod status_monitor;

use anyhow::Result;
use axum::{
    middleware,
    routing::get,
    Router,
};
use clap::Parser;
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber;

use crate::database::create_pool;
use crate::handlers_improved::create_router;
use crate::logging::{error_path_middleware, request_logging_middleware};
use crate::services::{ServerService, UserService};
use crate::status_monitor::{StatusMonitor, status_monitoring_middleware, get_status_report, get_health_check};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long)]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    let args = Args::parse();

    let database_url = args
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be set");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    info!("🚀 Starting Game Server Panel Backend");
    info!("📍 Server will bind to {}:{}", args.host, args.port);
    info!("🗄️ Database URL configured");
    info!("🔐 JWT secret configured");

    // Initialize status monitor
    let status_monitor = Arc::new(StatusMonitor::new());
    info!("📊 Status monitoring initialized");

    // Test database connection
    let pool = create_pool(&database_url).await?;
    info!("✅ Database connection pool created");

    // Skip migrations for now - database is set up manually
    info!("📋 Database setup completed");

    let user_service = Arc::new(UserService::new(pool.clone(), jwt_secret));
    let server_service = Arc::new(ServerService::new(pool));

    let app_state = Arc::new(crate::handlers_improved::AppState {
        user_service,
        server_service,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(["x-request-id", "x-response-time"]);

    // Build the application router
    let app = Router::new()
        // Root and health endpoints
        .route("/", get(root))
        .route("/health", get(get_health_check))
        .route("/status", get(get_status_report))
        
        // API routes
        .nest("/api/v1", create_router(app_state.clone()))
        
        // Status monitoring state
        .with_state(status_monitor.clone())
        
        // Middleware stack (order matters!)
        .layer(middleware::from_fn_with_state(
            status_monitor.clone(),
            status_monitoring_middleware,
        ))
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn(error_path_middleware))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("🌐 Server listening on {}", addr);
    info!("📖 Available endpoints:");
    info!("  GET  /           - API root");
    info!("  GET  /health     - Health check");
    info!("  GET  /status     - Detailed status report");
    info!("  POST /api/v1/auth/register - User registration");
    info!("  POST /api/v1/auth/login    - User login");
    info!("  GET  /api/v1/servers       - List servers");
    info!("  POST /api/v1/servers       - Create server");
    info!("  GET  /api/v1/servers/:id   - Get server details");
    info!("  POST /api/v1/servers/:id/start   - Start server");
    info!("  POST /api/v1/servers/:id/stop    - Stop server");
    info!("  POST /api/v1/servers/:id/restart - Restart server");
    info!("  GET  /api/v1/servers/:id/metrics - Server metrics");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": "Game Server Panel Backend",
        "version": "0.1.0",
        "description": "Backend API for managing game servers and VPS hosting",
        "status": "running",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "endpoints": {
            "health": "/health",
            "status": "/status",
            "api": "/api/v1",
            "documentation": "https://docs.example.com"
        },
        "features": [
            "User authentication",
            "Server management",
            "Real-time monitoring",
            "Status tracking",
            "Error logging",
            "Performance metrics"
        ]
    }))
}
