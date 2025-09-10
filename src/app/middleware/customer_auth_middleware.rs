use crate::config::jwt::verify_jwt;
use anyhow::Result;
use axum::{Json, extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde::Serialize;

/// A custom error response for the authentication middleware.
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub status: bool,
    pub message: String,
}

#[allow(dead_code)]
/// Middleware to protect customer routes.
/// It checks if the bearer token is valid and if the user's role is "User".
pub async fn customer_auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthErrorResponse>)> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    let token_string = if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            header.trim_start_matches("Bearer ").to_string()
        } else {
            let error_response = AuthErrorResponse {
                status: false,
                message: "Invalid token format. Bearer token expected.".to_owned(),
            };
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    } else {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Authorization header missing.".to_owned(),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    };

    // Verify the token and get the claims
    let token_data = verify_jwt(&token_string).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Invalid or expired token: {}", e),
        };
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    // Check if the user's role is "User"
    if token_data.claims.role != "User" {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Access denied. Only customers can view this page.".to_owned(),
        };
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    Ok(next.run(req).await)
}
