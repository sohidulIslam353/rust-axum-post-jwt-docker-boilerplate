use crate::models::user::Entity as User;
use axum::{Extension, Json, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;

// A serializable struct to format the JSON response for the user profile.
#[derive(Debug, Serialize)]
pub struct UserProfileData {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
}

// A serializable struct to format the JSON response for the user profile.
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub status: String,
    pub code: u16,
    pub data: Option<UserProfileData>,
}

pub async fn profile(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<UserProfileResponse>, (StatusCode, String)> {
    // Find the user with ID 1 in the database
    let user_model = User::find_by_id(1).one(&db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database Error: {}", e),
        )
    })?;

    if let Some(user_data) = user_model {
        // Create an instance of UserProfileData to exclude the password
        let response_data = UserProfileData {
            id: user_data.id,
            name: user_data.name,
            email: user_data.email,
            role: user_data.role,
        };

        let response = UserProfileResponse {
            status: "success".to_string(),
            code: 200,
            data: Some(response_data),
        };
        Ok(Json(response))
    } else {
        let response = UserProfileResponse {
            status: "error".to_string(),
            code: 404,
            data: None,
        };
        Ok(Json(response))
    }
}
