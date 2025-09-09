use super::redis::{delete_key, get_value, set_value};
use chrono::Utc;

#[allow(dead_code)]
pub async fn blacklist_token(token: &str, exp: usize) -> redis::RedisResult<()> {
    let ttl = exp as i64 - Utc::now().timestamp();
    if ttl > 0 {
        set_value(token, "blacklisted", ttl as usize).await?;
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn is_blacklisted(token: &str) -> redis::RedisResult<bool> {
    Ok(get_value(token).await?.is_some())
}

#[allow(dead_code)]
pub async fn remove_from_blacklist(token: &str) -> redis::RedisResult<()> {
    delete_key(token).await
}
