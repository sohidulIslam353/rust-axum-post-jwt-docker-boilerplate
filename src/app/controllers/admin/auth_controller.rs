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
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
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

    // Create JWT for the new user
    let token =
        create_jwt(&user.id.unwrap().to_string(), &"Admin".to_owned(), 3600).map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("JWT token not created: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    Ok(Json(AuthResponse {
        token,
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

        // Create JWT for the logged-in user with role "Admin"
        let token =
            create_jwt(&user_model.id.to_string(), &user_model.role, 3600).map_err(|e| {
                let error_response = AuthErrorResponse {
                    status: false,
                    message: format!("JWT token not created: {}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        Ok(Json(AuthResponse {
            token,
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

/// Handles the user logout logic by blacklisting the token.
pub async fn logout(
    Json(payload): Json<LogoutRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Decode the token to get its claims and expiration time
    let token_data = verify_jwt(&payload.token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("টোকেন অবৈধ: {}", e)))?;

    // Blacklist the token in Redis
    blacklist_token(&payload.token, token_data.claims.exp)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("টোকেন ব্ল্যাকলিস্ট করতে ব্যর্থ: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}
