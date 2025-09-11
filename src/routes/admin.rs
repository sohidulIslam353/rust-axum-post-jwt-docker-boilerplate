use axum::middleware::from_fn;
use axum::{Router, routing::delete, routing::get, routing::post, routing::put};

use crate::app::controllers::admin;
use crate::app::middleware::{admin_auth_middleware, admin_guest_middleware};

pub fn admin_routes() -> Router {
    // These routes are only accessible to unauthenticated (guest) users.
    let guest_routes = Router::new()
        .route("/login", post(admin::auth_controller::login))
        .route("/register", post(admin::auth_controller::register))
        .route(
            "/verify-email/:token",
            get(admin::auth_controller::verify_email),
        )
        .layer(from_fn(admin_guest_middleware::admin_guest_middleware));

    // These routes are accessible to any logged-in admin (token is valid and not blacklisted).
    let auth_routes = Router::new()
        .route(
            "/dashboard",
            get(admin::dashboard_controller::admin_dashboard),
        )
        .route(
            "/refresh-token",
            post(admin::auth_controller::refresh_token),
        )
        .route("/logout", post(admin::auth_controller::logout))
        .layer(from_fn(admin_auth_middleware::admin_auth_middleware));

    // The middleware is layered: first it checks for valid auth, then for email verification.
    let verified_routes = Router::new()
        .nest(
            "/categories",
            Router::new()
                .route("/", get(admin::category_controller::index))
                .route("/", post(admin::category_controller::store))
                .route("/:id", put(admin::category_controller::update))
                .route("/:id", delete(admin::category_controller::destroy)),
        )
        .layer(from_fn(admin_auth_middleware::admin_auth_middleware));

    Router::new()
        .merge(guest_routes)
        .merge(auth_routes)
        .merge(verified_routes)
}
