use axum::{Router, routing::delete, routing::get, routing::post, routing::put};

// এখানে আমরা একটি একক মডিউল থেকে সব হ্যান্ডলার ইম্পোর্ট করছি।
use crate::app::controllers::admin;

pub fn admin_routes() -> Router {
    Router::new()
        .route("/login", get(admin::auth_controller::login))
        .route("/register", post(admin::auth_controller::register))
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
}
