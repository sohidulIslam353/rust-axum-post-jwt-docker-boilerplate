use anyhow::Result;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

/// Middleware to check if a customer is already logged in and prevent them from
/// accessing certain routes like login and register.
#[allow(dead_code)]
pub async fn customer_guest_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Check if the request contains an "Authorization" header.
    if req.headers().contains_key("Authorization") {
        return Err(StatusCode::FORBIDDEN);
    }

    // If no authorization header, proceed to the next middleware or handler.
    Ok(next.run(req).await)
}
