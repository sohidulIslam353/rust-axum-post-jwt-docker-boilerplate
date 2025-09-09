use crate::models::user::Entity as User;
use axum::{Extension, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn profile(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<String, (StatusCode, String)> {
    // Find the first user in the database
    let first_user = User::find().one(&db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("ডাটাবেস ত্রুটি: {}", e),
        )
    })?;

    // Check if a user was found
    if let Some(user_model) = first_user {
        Ok(format!(
            "ডাটাবেস কানেকশন সফল! প্রথম ব্যবহারকারী '{}' পাওয়া গেছে।",
            user_model.name
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            "কোনো ব্যবহারকারী খুঁজে পাওয়া যায়নি।".to_string(),
        ))
    }
}
