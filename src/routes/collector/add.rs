use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{collector::Collector, user::User},
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AddCollectorPayload {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub id_number: String,
    pub phone_number: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub bank_name: String,
    pub bank_account_holder: String,
    pub bank_account_number: String,
}

#[utoipa::path(
    post,
    path = "/collector/add",
    request_body = AddCollectorPayload,
    tag = "Collector",
    security(("bearer_auth" = [])),
)]
pub async fn collector(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Json(payload): extract::Json<AddCollectorPayload>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business;

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

    let existing_collector = sqlx::query_as!(
        Collector,
        r#"
        SELECT * FROM collector_profile WHERE user_id = $1
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

    if existing_collector.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Bad Request",
                "reason": "User already has a collector profile.",
            })),
        ));
    }

    let collector = sqlx::query_as!(
        Collector,
        r#"
        INSERT INTO collector_profile (user_id, first_name, last_name, id_number, phone_number, address, city, state, zip_code, bank_name, bank_account_holder, bank_account_number)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING *
        "#,
        payload.user_id,
        payload.first_name,
        payload.last_name,
        payload.id_number,
        payload.phone_number,
        payload.address,
        payload.city,
        payload.state,
        payload.zip_code,
        payload.bank_name,
        payload.bank_account_holder,
        payload.bank_account_number
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
            "collector": collector,
        })),
    ))
}
