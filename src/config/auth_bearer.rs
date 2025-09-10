use crate::config::jwt::{JwtClaims, verify_jwt};
use anyhow::Result;
use axum::{
    Json, async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
// use jsonwebtoken::TokenData;
use serde::Serialize;

/// A custom extractor for JWT claims
pub struct AuthBearer(pub JwtClaims);

/// A custom error response for the authentication middleware.
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub status: bool,
    pub message: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthBearer
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<AuthErrorResponse>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok());

        let token_string = if let Some(header) = auth_header {
            if header.starts_with("Bearer ") {
                header.trim_start_matches("Bearer ").to_string()
            } else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        status: false,
                        message: "Invalid token format. Bearer token expected.".to_owned(),
                    }),
                ));
            }
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    status: false,
                    message: "Authorization header missing.".to_owned(),
                }),
            ));
        };

        // Verify the token and get the claims
        let token_data = verify_jwt(&token_string).map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    status: false,
                    message: format!("Invalid or expired token: {}", e),
                }),
            )
        })?;

        Ok(AuthBearer(token_data.claims))
    }
}
