use axum::{Router, routing::delete, routing::get, routing::post, routing::put};

// Import the controller
use crate::app::controllers::admin::auth_controller::admin_login;
use crate::app::controllers::admin::category_controller::{destroy, index, store, update};
use crate::app::controllers::admin::dashboard_controller::admin_dashboard;

pub fn admin_routes() -> Router {
    // Basic admin routes
    let admin_base = Router::new()
        .route("/login", get(admin_login))
        .route("/dashboard", get(admin_dashboard));

    // Category CRUD routes under / categories prefix
    let category_routes = Router::new()
        .route("/", get(index))
        .route("/", post(store))
        .route("/:id", put(update))
        .route("/:id", delete(destroy));

    // Merge category routes with /categories prefix into admin router
    admin_base.nest("/categories", category_routes)
}
