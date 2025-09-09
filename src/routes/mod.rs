use axum::{Router, routing::get};

pub mod admin;
pub mod customer;

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .nest("/admin", admin::admin_routes())
        .nest("/customer", customer::customer_routes())
}
