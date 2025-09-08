use axum::{Router, routing::get};

pub fn customer_routes() -> Router {
    Router::new().route("/customer", get(customer_index))
}

async fn customer_index() -> &'static str {
    "Hello from Customer!"
}
