use once_cell::sync::OnceCell;
use redis::AsyncCommands;
use redis::Client;
use redis::aio::MultiplexedConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared cache type
pub type SharedCache = Arc<Mutex<MultiplexedConnection>>;

/// Global cache instance
static CACHE: OnceCell<SharedCache> = OnceCell::new();

/// Initialize Redis/KeyDB client
pub async fn init_cache(redis_url: &str) -> SharedCache {
    let client = Client::open(redis_url).expect("Invalid Redis URL");
    let conn = client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to connect to Redis/KeyDB");

    let shared = Arc::new(Mutex::new(conn));
    CACHE.set(shared.clone()).ok(); // store globally
    shared
}

/// Get global cache
pub fn get_cache() -> SharedCache {
    CACHE
        .get()
        .expect("Cache not initialized. Call init_cache first.")
        .clone()
}

/// Set a value in cache with TTL (seconds)
pub async fn set_value(key: &str, value: &str, ttl_seconds: usize) -> redis::RedisResult<()> {
    let cache = get_cache();
    let mut conn = cache.lock().await;
    conn.set_ex(key, value, ttl_seconds as u64).await
}

/// Get a value from cache
pub async fn get_value(key: &str) -> redis::RedisResult<Option<String>> {
    let cache = get_cache();
    let mut conn = cache.lock().await;
    conn.get(key).await
}

/// Delete a key from cache
#[allow(dead_code)]
pub async fn delete_key(key: &str) -> redis::RedisResult<()> {
    let cache = get_cache();
    let mut conn = cache.lock().await;
    conn.del(key).await
}
