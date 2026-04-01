mod auth;
mod database;
mod handlers;
mod middleware;
mod models;
mod services;

use anyhow::Result;
use axum::{
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

use crate::database::{create_pool, run_migrations};
use crate::handlers::{create_router, AppState};
use crate::services::{ServerService, UserService};

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
        .init();

    let args = Args::parse();

    let database_url = args
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be set");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    info!("Starting server on {}:{}", args.host, args.port);

    let pool = create_pool(&database_url).await?;
    info!("Database connection pool created");

    // Skip migrations for now - database is set up manually
    // run_migrations(&pool).await?;
    info!("Database setup completed");

    let user_service = Arc::new(UserService::new(pool.clone(), jwt_secret));
    let server_service = Arc::new(ServerService::new(pool));

    let app_state = Arc::new(AppState {
        user_service,
        server_service,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Game Server Panel API" }))
        .nest("/api/v1", create_router(app_state))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
