use crate::config::cache;
use axum::Json;
pub async fn admin_login() -> Json<String> {
    let cache_key = "categories";

    // 1️⃣ Try to get from cache
    if let Ok(Some(cached)) = cache::get_value(cache_key).await {
        println!("Cache hit!");
        return Json(cached); // return cached value
    }

    Json("Hello Admin".to_string())
}
