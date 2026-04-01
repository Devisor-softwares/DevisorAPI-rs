use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::auth::generate_token;
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
        .route("/servers/:id", get(get_server).put(update_server).delete(delete_server))
        .route("/servers/:id/start", post(start_server))
        .route("/servers/:id/stop", post(stop_server))
        .route("/servers/:id/restart", post(restart_server))
        .route("/servers/:id/metrics", get(get_server_metrics))
        .with_state(state)
}

pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    if let Err(_) = request.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = state
        .user_service
        .create_user(request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = generate_token(&user, &state.user_service.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        token,
        user,
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let (user, token) = state
        .user_service
        .login(request)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(Json(AuthResponse {
        token,
        user,
    }))
}

pub async fn get_servers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Server>>, StatusCode> {
    // TODO: Add authentication middleware to get user_id
    let user_id = Uuid::new_v4(); // Placeholder

    let servers = state
        .server_service
        .get_servers_by_user(user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(servers))
}

pub async fn create_server(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateServerRequest>,
) -> Result<Json<Server>, StatusCode> {
    // TODO: Add authentication middleware to get user_id
    let user_id = Uuid::new_v4(); // Placeholder

    if let Err(_) = request.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let server = state
        .server_service
        .create_server(request, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(server))
}

pub async fn get_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Server>, StatusCode> {
    let server = state
        .server_service
        .get_server_by_id(server_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match server {
        Some(server) => Ok(Json(server)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(request): Json<UpdateServerRequest>,
) -> Result<Json<Server>, StatusCode> {
    if let Err(_) = request.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let server = state
        .server_service
        .update_server(server_id, request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(server))
}

pub async fn delete_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .server_service
        .delete_server(server_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let command = ServerCommand {
        server_id,
        command: ServerCommandType::Start,
        parameters: None,
    };

    state
        .server_service
        .execute_server_command(command)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::ACCEPTED)
}

pub async fn stop_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let command = ServerCommand {
        server_id,
        command: ServerCommandType::Stop,
        parameters: None,
    };

    state
        .server_service
        .execute_server_command(command)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::ACCEPTED)
}

pub async fn restart_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let command = ServerCommand {
        server_id,
        command: ServerCommandType::Restart,
        parameters: None,
    };

    state
        .server_service
        .execute_server_command(command)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::ACCEPTED)
}

pub async fn get_server_metrics(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<ServerMetrics>>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let metrics = state
        .server_service
        .get_server_metrics(server_id, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metrics))
}
