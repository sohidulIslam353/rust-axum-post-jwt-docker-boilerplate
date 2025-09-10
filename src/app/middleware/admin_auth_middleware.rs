use crate::config::blacklist::is_blacklisted;
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

/// Middleware to protect admin routes.
/// It checks if the bearer token is valid and if the user's role is "Admin".
#[allow(dead_code)]
pub async fn admin_auth_middleware(
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

    // Check if the token is blacklisted after successful verification
    if is_blacklisted(&token_string).await.map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Failed to check token blacklist: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })? {
        let error_response = AuthErrorResponse {
            status: false,
            message: "This token has been blacklisted.".to_owned(),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    // Check if the user's role is "Admin"
    if token_data.claims.role != "Admin" {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Access denied. Only administrators can view this page.".to_owned(),
        };
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    Ok(next.run(req).await)
}
