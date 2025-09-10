use crate::config::blacklist::is_blacklisted;
use crate::config::jwt::verify_jwt;
use axum::{Json, extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde::Serialize;

/// A custom error response for the authentication middleware.
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub status: bool,
    pub message: String,
}

/// Middleware to prevent authenticated users from accessing guest routes like login or register.
pub async fn admin_guest_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthErrorResponse>)> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            let token_string = header.trim_start_matches("Bearer ").to_string();

            let is_valid_token = verify_jwt(&token_string).is_ok();
            let is_token_blacklisted = is_blacklisted(&token_string).await.unwrap_or(false);

            if is_valid_token && !is_token_blacklisted {
                let error_response = AuthErrorResponse {
                    status: false,
                    message: "Access denied. You are already logged in.".to_owned(),
                };
                return Err((StatusCode::FORBIDDEN, Json(error_response)));
            }
        }
    }

    // Continue to the next middleware or handler if no valid token is found.
    Ok(next.run(req).await)
}
