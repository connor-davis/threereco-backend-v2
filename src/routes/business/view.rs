use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{business::Business, user::User},
    AppState,
};

#[utoipa::path(
    get,
    path = "/business",
    tag = "Business",
    security(("bearer_auth" = [])),
)]
pub async fn businesses(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    if authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

    let businesses = sqlx::query_as!(Business, r#"SELECT * FROM business_profile"#)
        .fetch_all(&app_state.pool)
        .await
        .map_err(|error| {
            tracing::error!("🔥 Error while fetching businesses: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal Server Error",
                    "reason": "Unknown error occured. Please contact the api developer."
                })),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "businesses": businesses
        })),
    ))
}

#[utoipa::path(
    get,
    path = "/business/{business_id}",
    params(("business_id" = String, Path, description = "The businesses id.")),
    tag = "Business",
    security(("bearer_auth" = [])),
)]
pub async fn business(
    extract::State(app_state): extract::State<AppState>,
    extract::Path(business_id): extract::Path<Uuid>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a =
        authenticated_user.role() != Role::Staff && authenticated_user.role() != Role::SystemAdmin;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

    let business = sqlx::query_as!(
        Business,
        r#"
        SELECT * FROM business_profile WHERE id = $1
        "#,
        business_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Failed to query database: {}", error);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Internal Server Error",
                "reason": "Failed to query database."
            })),
        )
    })?;

    if business.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "reason": "Business not found."
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "business": business.unwrap()
        })),
    ))
}
