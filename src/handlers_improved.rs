use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::auth::generate_token;
use crate::logging::{ApiResponse, ApiError, error_responses};
use crate::models::{
    AuthResponse, CreateServerRequest, CreateUserRequest, LoginRequest, Server, ServerCommand,
    ServerCommandType, ServerMetrics, UpdateServerRequest,
};
use crate::services::{ServerService, UserService};

pub struct AppState {
    pub user_service: Arc<UserService>,
    pub server_service: Arc<ServerService>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/servers", get(get_servers).post(create_server))
        .route("/servers/:id", get(get_server))
        .route("/servers/:id/start", post(start_server))
        .route("/servers/:id/stop", post(stop_server))
        .route("/servers/:id/restart", post(restart_server))
        .route("/servers/:id/metrics", get(get_server_metrics))
        .with_state(state)
}

pub async fn health_check() -> ApiResponse {
    ApiResponse::new()
        .message("Service is healthy")
        .meta("service", "game-server-panel")
        .meta("version", "0.1.0")
        .meta("timestamp", chrono::Utc::now().to_rfc3339())
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<ApiResponse, ApiError> {
    if let Err(errors) = request.validate() {
        return Err(error_responses::validation_error(format!("Validation failed: {:?}", errors)));
    }

    let user = state
        .user_service
        .create_user(request)
        .await
        .map_err(|e| error_responses::internal_error(format!("Failed to create user: {}", e)))?;

    let token = generate_token(&user, &state.user_service.jwt_secret)
        .map_err(|e| error_responses::internal_error(format!("Failed to generate token: {}", e)))?;

    Ok(ApiResponse::new()
        .message("User registered successfully")
        .data(AuthResponse {
            token,
            user,
        }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<ApiResponse, ApiError> {
    let (user, token) = state
        .user_service
        .login(request)
        .await
        .map_err(|_| error_responses::auth_error("Invalid credentials"))?;

    Ok(ApiResponse::new()
        .message("Login successful")
        .data(AuthResponse {
            token,
            user,
        }))
}

pub async fn get_servers(
    State(state): State<Arc<AppState>>,
) -> Result<ApiResponse, ApiError> {
    // TODO: Add authentication middleware to get user_id
    let user_id = Uuid::new_v4(); // Placeholder

    let servers = state
        .server_service
        .get_servers_by_user(user_id)
        .await
        .map_err(|e| error_responses::database_error(format!("Failed to fetch servers: {}", e)))?;

    Ok(ApiResponse::new()
        .message("Servers retrieved successfully")
        .data(servers)
        .meta("count", servers.len()))
}

pub async fn create_server(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateServerRequest>,
) -> Result<ApiResponse, ApiError> {
    if let Err(errors) = request.validate() {
        return Err(error_responses::validation_error(format!("Validation failed: {:?}", errors)));
    }

    // TODO: Add authentication middleware to get user_id
    let user_id = Uuid::new_v4(); // Placeholder

    let server = state
        .server_service
        .create_server(request, user_id)
        .await
        .map_err(|e| error_responses::database_error(format!("Failed to create server: {}", e)))?;

    Ok(ApiResponse::created()
        .data(server)
        .meta("server_id", server.id))
}

pub async fn get_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<ApiResponse, ApiError> {
    let server = state
        .server_service
        .get_server_by_id(server_id)
        .await
        .map_err(|e| error_responses::database_error(format!("Failed to fetch server: {}", e)))?;

    match server {
        Some(server) => Ok(ApiResponse::new()
            .message("Server retrieved successfully")
            .data(server)),
        None => Err(error_responses::not_found("Server")),
    }
}

pub async fn start_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<ApiResponse, ApiError> {
    let command = ServerCommand {
        server_id,
        command: ServerCommandType::Start,
        parameters: None,
    };

    state
        .server_service
        .execute_server_command(command)
        .await
        .map_err(|e| error_responses::internal_error(format!("Failed to start server: {}", e)))?;

    Ok(ApiResponse::new()
        .message("Server start command sent successfully")
        .meta("server_id", server_id)
        .meta("action", "start"))
}

pub async fn stop_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<ApiResponse, ApiError> {
    let command = ServerCommand {
        server_id,
        command: ServerCommandType::Stop,
        parameters: None,
    };

    state
        .server_service
        .execute_server_command(command)
        .await
        .map_err(|e| error_responses::internal_error(format!("Failed to stop server: {}", e)))?;

    Ok(ApiResponse::new()
        .message("Server stop command sent successfully")
        .meta("server_id", server_id)
        .meta("action", "stop"))
}

pub async fn restart_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<ApiResponse, ApiError> {
    let command = ServerCommand {
        server_id,
        command: ServerCommandType::Restart,
        parameters: None,
    };

    state
        .server_service
        .execute_server_command(command)
        .await
        .map_err(|e| error_responses::internal_error(format!("Failed to restart server: {}", e)))?;

    Ok(ApiResponse::new()
        .message("Server restart command sent successfully")
        .meta("server_id", server_id)
        .meta("action", "restart"))
}

pub async fn get_server_metrics(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<ApiResponse, ApiError> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let metrics = state
        .server_service
        .get_server_metrics(server_id, limit)
        .await
        .map_err(|e| error_responses::database_error(format!("Failed to fetch metrics: {}", e)))?;

    Ok(ApiResponse::new()
        .message("Server metrics retrieved successfully")
        .data(metrics)
        .meta("server_id", server_id)
        .meta("limit", limit)
        .meta("count", metrics.len()))
}
