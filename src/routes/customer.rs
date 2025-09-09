use axum::{Router, routing::get};

// এখানে আমরা একটি একক মডিউল থেকে সব হ্যান্ডলার ইম্পোর্ট করছি।
use crate::app::controllers::customer;
pub fn customer_routes() -> Router {
    Router::new().route("/profile", get(customer::auth_controller::profile))
}
