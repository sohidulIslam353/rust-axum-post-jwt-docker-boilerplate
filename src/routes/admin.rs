use axum::{Router, routing::delete, routing::get, routing::post, routing::put};

use crate::app::controllers::admin;
use crate::app::middleware::{admin_auth_middleware, admin_guest_middleware};
use axum::middleware;

pub fn admin_routes() -> Router {
    // For all guest routes
    let guest_routes = Router::new()
        .route("/login", get(admin::auth_controller::login))
        .route("/register", post(admin::auth_controller::register))
        .layer(middleware::from_fn(
            admin_guest_middleware::admin_guest_middleware,
        ));

    // Admin Routes
    let auth_routes = Router::new()
        .route(
            "/refresh-token",
            post(admin::auth_controller::refresh_token),
        )
        .route("/logout", post(admin::auth_controller::logout))
        .route(
            "/dashboard",
            get(admin::dashboard_controller::admin_dashboard),
        )
        .nest(
            "/categories",
            Router::new()
                .route("/", get(admin::category_controller::index))
                .route("/", post(admin::category_controller::store))
                .route("/:id", put(admin::category_controller::update))
                .route("/:id", delete(admin::category_controller::destroy)),
        )
        .layer(middleware::from_fn(
            admin_auth_middleware::admin_auth_middleware,
        ));

    Router::new().merge(guest_routes).merge(auth_routes)
}
