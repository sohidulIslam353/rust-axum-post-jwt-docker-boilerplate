use axum::{Extension, Json, http::StatusCode};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::config::blacklist::blacklist_token;
use crate::config::jwt::{create_jwt, verify_jwt};
use crate::models::{user, user::Entity as User};
use bcrypt::verify;

/// A struct to represent the user registration request body.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// A struct to represent the response after successful login or registration.
/// This is updated to include both an access token and a refresh token.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

/// A struct to represent a custom error response.
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub status: bool,
    pub message: String,
}

/// Handles the admin registration logic.
pub async fn register(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    // Check if user with this email already exists
    let existing_user = User::find()
        .filter(user::Column::Email.eq(payload.email.clone()))
        .one(&db)
        .await
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Database error: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if existing_user.is_some() {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Email already exists.".to_string(),
        };
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    // Hash the password
    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Password hashing error: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Create a new user active model with role "Admin"
    let new_user = user::ActiveModel {
        name: Set(payload.name),
        email: Set(payload.email),
        password: Set(hashed_password),
        role: Set("Admin".to_owned()),
        ..Default::default()
    };

    // Save the user to the database
    let user = new_user.save(&db).await.map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("has an issue to insert data into database: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Get the user ID and store it to avoid ownership move errors
    let user_id = user.id.unwrap().to_string();

    // Create a new access token (1 hour) and a refresh token (30 days).
    let access_token = create_jwt(&user_id, &"Admin".to_owned(), 3600).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Access token not created: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let refresh_token = create_jwt(&user_id, &"Admin".to_owned(), 2592000).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Refresh token not created: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_owned(),
    }))
}

/// A struct to represent the user login request body.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Handles the admin login logic.
pub async fn login(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    // Find the user by email
    let user_model = User::find()
        .filter(user::Column::Email.eq(payload.email))
        .one(&db)
        .await
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Database error: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if let Some(user_model) = user_model {
        // Verify the password
        let password_is_valid = verify(payload.password, &user_model.password).map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Password verification error: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

        if !password_is_valid {
            let error_response = AuthErrorResponse {
                status: false,
                message: "Wrong email or password.".to_string(),
            };
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }

        // Create a new access token and a new refresh token.
        let access_token =
            create_jwt(&user_model.id.to_string(), &user_model.role, 3600).map_err(|e| {
                let error_response = AuthErrorResponse {
                    status: false,
                    message: format!("Access token not created: {}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        let refresh_token = create_jwt(&user_model.id.to_string(), &user_model.role, 2592000)
            .map_err(|e| {
                let error_response = AuthErrorResponse {
                    status: false,
                    message: format!("Refresh token not created: {}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        Ok(Json(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_owned(),
        }))
    } else {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Wrong email or password.".to_string(),
        };
        Err((StatusCode::UNAUTHORIZED, Json(error_response)))
    }
}

/// A struct to represent the request to logout.
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub token: String,
}

/// A struct to represent a successful logout response.
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub status: bool,
    pub message: String,
}

/// Handles the user logout logic by blacklisting the token.
/// This is the best practice for revoking JWTs before they expire.
pub async fn logout(
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    // Decode the token to get its claims and expiration time
    let token_data = verify_jwt(&payload.token).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Invalid Token: {}", e),
        };
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    // Blacklist the token in Redis
    blacklist_token(&payload.token, token_data.claims.exp)
        .await
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Token not blacklisted: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    Ok(Json(LogoutResponse {
        status: true,
        message: "Logout successful.".to_owned(),
    }))
}

/// A struct to represent the refresh token request body.
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Handles the token refresh logic.
pub async fn refresh_token(
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    // 1. Verify the refresh token's validity.
    let token_data = verify_jwt(&payload.refresh_token).map_err(|_| {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Invalid or expired refresh token.".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    // 2. Check if the token is blacklisted.
    let is_blacklisted = crate::config::redis::key_exists(&payload.refresh_token)
        .await
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Failed to check token blacklist: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if is_blacklisted {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Refresh token is blacklisted.".to_string(),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    // 3. Blacklist the old refresh token to prevent reuse.
    blacklist_token(&payload.refresh_token, token_data.claims.exp)
        .await
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Failed to blacklist old refresh token: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    // 4. Generate a new access token (1 hour) and a new refresh token (30 days).
    let new_access_token = create_jwt(&token_data.claims.id, &token_data.claims.role, 3600)
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Failed to create new access token: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let new_refresh_token = create_jwt(&token_data.claims.id, &token_data.claims.role, 2592000)
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Failed to create new refresh token: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    // 5. Return the new tokens.
    Ok(Json(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_owned(),
    }))
}
