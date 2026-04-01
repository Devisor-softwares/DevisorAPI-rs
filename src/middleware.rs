use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::handlers::AppState;
use crate::models::User;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            Some(&header[7..])
        }
        _ => None,
    };

    let token = token.ok_or_else(|| {
        tracing::warn!("Missing or invalid authorization header");
        StatusCode::UNAUTHORIZED
    })?;

    let user = state
        .user_service
        .verify_token(token)
        .await
        .map_err(|e| {
            tracing::warn!("Token verification failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // Add user to request extensions for handlers to use
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

pub async fn admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = request.extensions().get::<User>().ok_or_else(|| {
        tracing::warn!("User not found in request extensions");
        StatusCode::UNAUTHORIZED
    })?;

    match user.role {
        crate::models::UserRole::Admin => Ok(next.run(request).await),
        crate::models::UserRole::User => {
            tracing::warn!("Access denied for non-admin user: {}", user.username);
            Err(StatusCode::FORBIDDEN)
        }
    }
}
