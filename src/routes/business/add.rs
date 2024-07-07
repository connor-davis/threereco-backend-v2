use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{business::Business, user::User},
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AddBusinessPayload {
    pub user_id: Uuid,
    pub business_name: String,
    pub business_type: String,
    pub business_description: String,
    pub phone_number: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
}

#[utoipa::path(
    post,
    path = "/business/add",
    request_body = AddBusinessPayload,
    tag = "Business",
    security(("bearer_auth" = [])),
)]
pub async fn business(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Json(payload): extract::Json<AddBusinessPayload>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a =
        authenticated_user.role() != Role::Staff && authenticated_user.role() != Role::SystemAdmin;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(
                json!({ "error": "Unauthorized", "reason": "You do not have permission to add users." }),
            ),
        ));
    }

    let existing_user = sqlx::query!(
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
        payload.user_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Failed to query database: {}", error);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                json!({ "error": "Internal Server Error", "reason": "Failed to query database." }),
            ),
        )
    })?;

    if existing_user.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Bad Request",
                "reason": "User does not exist.",
            })),
        ));
    }

    let existing_business = sqlx::query_as!(
        Business,
        r#"
        SELECT * FROM business_profile WHERE user_id = $1
        "#,
        payload.user_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Failed to query database: {}", error);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                json!({ "error": "Internal Server Error", "reason": "Failed to query database." }),
            ),
        )
    })?;

    if existing_business.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Bad Request",
                "reason": "User already has a business profile.",
            })),
        ));
    }

    let business = sqlx::query_as!(
        Business,
        r#"
        INSERT INTO business_profile (user_id, business_name, business_type, business_description, phone_number, address, city, state, zip_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
        payload.user_id,
        payload.business_name,
        payload.business_type,
        payload.business_description,
        payload.phone_number,
        payload.address,
        payload.city,
        payload.state,
        payload.zip_code
    )
    .fetch_one(&app_state.pool)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Failed to query database: {}", error);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                json!({ "error": "Internal Server Error", "reason": "Failed to query database." }),
            ),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "business": business,
        })),
    ))
}
