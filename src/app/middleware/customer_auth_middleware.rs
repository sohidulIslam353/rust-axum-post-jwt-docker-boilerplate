use crate::{
    app::controllers::admin::auth_controller::AuthErrorResponse, // controller's error type
    config::{
        auth_bearer::{AuthBearer, AuthErrorResponse as BearerAuthError}, // alias the bearer error
        blacklist::is_blacklisted,
    },
};
use axum::{
    Json,
    extract::FromRequestParts,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Middleware to protect customer routes.
/// It checks if the bearer token is valid, not blacklisted, and if the user's role is "User".
#[allow(dead_code)]
pub async fn customer_auth_middleware(
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthErrorResponse>)> {
    // split request into parts + body
    let (mut parts, body) = req.into_parts();

    // call extractor but DON'T use `?` — map the Err from bearer->controller error
    let auth_bearer = match AuthBearer::from_request_parts(&mut parts, &()).await {
        Ok(ab) => ab,
        Err((status, json)) => {
            // json: axum::Json<BearerAuthError>
            let be: BearerAuthError = json.0;

            // convert to controller's AuthErrorResponse
            let converted = AuthErrorResponse {
                status: be.status,
                message: be.message,
            };

            return Err((status, Json(converted)));
        }
    };

    // rebuild request for downstream handlers
    let req = Request::from_parts(parts, body);

    let claims = auth_bearer.0;

    // Check if the token is blacklisted
    if is_blacklisted(&claims.sub).await.map_err(|e| {
        let error_response = AuthErrorResponse {
            status: false,
            message: format!("Failed to check token blacklist: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })? {
        let error_response = AuthErrorResponse {
            status: false,
            message: "This token has been blacklisted.".to_owned(),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    // Check if the user's role is "User"
    if claims.role != "User" {
        let error_response = AuthErrorResponse {
            status: false,
            message: "Access denied. Only customers can view this page.".to_owned(),
        };
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    Ok(next.run(req).await)
}
