use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;

use crate::data::entities::user::User;

#[utoipa::path(
    get,
    path = "/authentication/check",
    tag = "Authentication",
    responses(
        (
            status = 200,
            content_type = "application/json",
            description = "Successful check.",
        ),
        (
            status = 401,
            content_type = "application/json",
            description = "Unauthorized. User does not exist/Invalid token/Expired token.",
        ),
        (
            status = 500,
            content_type = "application/json",
            description = "Internal Server Error. Please contact the developer.",
        ),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn index(
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    Ok((StatusCode::OK, Json(authenticated_user)))
}
