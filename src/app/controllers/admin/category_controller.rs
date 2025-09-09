use crate::config::redis;
use axum::Json;

pub async fn index() -> Json<String> {
    let cache_key = "categories";

    // 1️⃣ Try to get from cache
    if let Ok(Some(cached)) = redis::get_value(cache_key).await {
        println!("Cache hit!");
        return Json(cached); // return cached value
    }

    // 2️⃣ If not in cache, fetch from "DB" (here we use static example)
    let categories = vec!["Electronics", "Books", "Clothing"];
    let value = serde_json::to_string(&categories).unwrap();

    // 3️⃣ Set value in cache for 60 seconds
    let _ = redis::set_value(cache_key, &value, 600).await;

    println!("Cache miss! Setting cache.");
    Json(value)
}

pub async fn store() -> &'static str {
    "Category Store"
}

pub async fn update() -> &'static str {
    "Admin Update"
}

pub async fn destroy() -> &'static str {
    "Admin Delete"
}
