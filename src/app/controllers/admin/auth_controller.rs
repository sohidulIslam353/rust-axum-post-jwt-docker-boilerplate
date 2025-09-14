use anyhow::Result;
use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::config::blacklist::blacklist_token;
use crate::config::jwt::{
    create_email_verification_jwt, create_jwt, verify_email_verification_jwt, verify_jwt,
};
// use crate::config::mail::EmailSender;
use crate::config::rabbitmq::EmailJob;
use crate::config::rabbitmq::publish_to_queue; // Import publish_to_queue
use crate::models::{user, user::Entity as User};

use bcrypt::verify;
use chrono::Utc;
use tracing::{error, info};

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

/// A struct to represent a successful verification response.
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
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
        error!(
            "Registration failed: Email already exists for user: {}",
            payload.email
        );
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
        email: Set(payload.email.clone()),
        password: Set(hashed_password),
        role: Set("Admin".to_owned()),
        // `email_verified_at` is set to null by default
        ..Default::default()
    };

    // Save the user to the database
    let user = new_user.save(&db).await.map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Database issue: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Get the user ID
    let user_id = user.id.unwrap().to_string();

    // Create JWT tokens (access and refresh)
    let access_token = create_jwt(&user_id, &"Admin".to_owned(), 3600).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Access token creation failed: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let refresh_token = create_jwt(&user_id, &"Admin".to_owned(), 2592000).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Refresh token creation failed: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Create a verification token and send it to RabbitMQ
    let verification_token = create_email_verification_jwt(&user_id, 86400).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Verification token creation failed: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let verification_link = format!(
        "http://localhost:8080/admin/verify-email/{}",
        verification_token
    );

    // Create the email task (email job) to be processed in the background
    let email_task = EmailJob {
        to: payload.email.clone(),
        subject: "Verify your email address".to_string(),
        body: format!(
            "<html><body><h1>ইমেল যাচাই করুন</h1><p>আপনার অ্যাকাউন্ট যাচাই করতে নিচের লিংকে ক্লিক করুন:</p><a href=\"{}\">ইমেল যাচাই করুন</a></body></html>",
            verification_link
        ),
    };

    // Publish the email task to the RabbitMQ queue
    publish_to_queue(&email_task, "email_queue")
        .await
        .map_err(|e| {
            let error_response = AuthErrorResponse {
                status: false,
                message: format!("Failed to send email task to RabbitMQ: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    info!("Verification email send to: {}", payload.email);
    // Return the authentication tokens
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

/// Handles the email verification logic.
pub async fn verify_email(
    Extension(db): Extension<DatabaseConnection>,
    Path(token): Path<String>,
) -> Result<Json<VerifyResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    //  Verify token and get claims
    let token_data = verify_email_verification_jwt(&token).map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Invalid or expired verification token: {}", e),
        };
        (StatusCode::BAD_REQUEST, Json(error_response))
    })?;

    //  Convert string user_id to i32 (DB id)
    let user_id: i32 = token_data.claims.id.parse().map_err(|_| {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Invalid user ID in token.".to_string(),
        };
        (StatusCode::BAD_REQUEST, Json(error_response))
    })?;

    //  Fetch user from DB
    let user_model = User::find_by_id(user_id).one(&db).await.map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Database error: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_model = match user_model {
        Some(u) => u,
        None => {
            let error_response = AuthErrorResponse {
                status: false,
                message: "User not found.".to_string(),
            };
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    };

    //  Check if already verified
    if user_model.email_verified_at.is_some() {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Email is already verified.".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    //  Update email_verified_at
    let mut user_active_model: user::ActiveModel = user_model.into();
    user_active_model.email_verified_at = Set(Some(Utc::now().naive_utc()));
    user_active_model.save(&db).await.map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Failed to update email verification status: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    //  Success response
    Ok(Json(VerifyResponse {
        status: true,
        message: "Email successfully verified.".to_owned(),
    }))
}
