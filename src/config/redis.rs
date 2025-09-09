use anyhow::{Result, anyhow};
use once_cell::sync::OnceCell;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisResult};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared cache type
pub type SharedCache = Arc<Mutex<MultiplexedConnection>>;

/// Global cache instance
static CACHE: OnceCell<SharedCache> = OnceCell::new();

/// Initialize Redis/KeyDB client and store globally
pub async fn init_cache(redis_url: &str) -> Result<SharedCache> {
    let client = Client::open(redis_url).map_err(|e| anyhow!("Invalid Redis URL: {}", e))?;

    let conn = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| anyhow!("Failed to connect to Redis/KeyDB: {}", e))?;

    let shared = Arc::new(Mutex::new(conn));
    CACHE.set(shared.clone()).ok();
    Ok(shared)
}

/// Get global cache instance
pub fn get_cache() -> SharedCache {
    CACHE
        .get()
        .expect("Cache not initialized. Call init_cache first.")
        .clone()
}

/// Set a value in cache with TTL (seconds)
pub async fn set_value(key: &str, value: &str, ttl_seconds: usize) -> RedisResult<()> {
    let cache = get_cache();
    let mut conn = cache.lock().await;
    conn.set_ex(key, value, ttl_seconds as u64).await
}

/// Get a value from cache
pub async fn get_value(key: &str) -> RedisResult<Option<String>> {
    let cache = get_cache();
    let mut conn = cache.lock().await;
    conn.get(key).await
}

#[allow(dead_code)]
/// Delete a key from cache
pub async fn delete_key(key: &str) -> RedisResult<()> {
    let cache = get_cache();
    let mut conn = cache.lock().await;
    conn.del(key).await
}
#[allow(dead_code)]
/// Check if a key exists in cache
pub async fn key_exists(key: &str) -> RedisResult<bool> {
    let cache = get_cache();
    let mut conn = cache.lock().await;
    conn.exists(key).await
}
