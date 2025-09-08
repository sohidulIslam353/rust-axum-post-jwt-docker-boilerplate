pub mod admin;
pub mod customer;

use axum::{Router, routing::get};

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .nest("/admin", admin::admin_routes()) // set prefix for all admin routes
        .merge(customer::customer_routes())
}
