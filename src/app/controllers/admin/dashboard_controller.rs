use axum::{Json, http::StatusCode};
use serde::Serialize;

use crate::config::auth_bearer::AuthBearer;

/// A struct to represent a custom error response.
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub status: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AdminDashboardResponse {
    pub message: String,
    pub user_id: String,
}

/// Handles the admin dashboard logic.
pub async fn admin_dashboard(
    claims: AuthBearer,
) -> Result<Json<AdminDashboardResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    if claims.0.role != "Admin" {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Access denied. Only administrators can view this page.".to_string(),
        };
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    // You can now use the `claims.sub` (user ID) to fetch specific user data.
    Ok(Json(AdminDashboardResponse {
        message: "Welcome to the admin dashboard!".to_owned(),
        user_id: claims.0.sub,
    }))
}
