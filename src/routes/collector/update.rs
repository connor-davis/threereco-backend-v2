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
pub struct UpdateCollectorPayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub id_number: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_holder: Option<String>,
    pub bank_account_number: Option<String>,
}

#[utoipa::path(
    post,
    path = "/collector/{collector_id}",
    params(("collector_id" = String, Path, description = "The collectors id.")),
    request_body = UpdateCollectorPayload,
    tag = "Collector",
    security(("bearer_auth" = [])),
)]
pub async fn collector(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(collector_id): extract::Path<Uuid>,
    extract::Json(payload): extract::Json<UpdateCollectorPayload>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to update user data."
            })),
        ));
    }

    let existing_collector = sqlx::query_as!(
        Collector,
        r#"
        SELECT * FROM collector_profile WHERE id = $1
        "#,
        collector_id
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

    if existing_collector.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "reason": "Collector not found."
            })),
        ));
    }

    let collector = existing_collector.unwrap();

    let first_name = payload.first_name.unwrap_or(collector.first_name);
    let last_name = payload.last_name.unwrap_or(collector.last_name);
    let id_number = payload.id_number.unwrap_or(collector.id_number);
    let phone_number = payload.phone_number.unwrap_or(collector.phone_number);
    let address = payload.address.unwrap_or(collector.address);
    let city = payload.city.unwrap_or(collector.city);
    let state = payload.state.unwrap_or(collector.state);
    let zip_code = payload.zip_code.unwrap_or(collector.zip_code);
    let bank_name = payload.bank_name.unwrap_or(collector.bank_name);
    let bank_account_holder = payload
        .bank_account_holder
        .unwrap_or(collector.bank_account_holder);
    let bank_account_number = payload
        .bank_account_number
        .unwrap_or(collector.bank_account_number);

    let collector = sqlx::query_as!(
        Collector,
        r#"
        UPDATE collector_profile
        SET first_name = $1, last_name = $2, id_number = $3, phone_number = $4, address = $5, city = $6, state = $7, zip_code = $8, bank_name = $9, bank_account_holder = $10, bank_account_number = $11
        WHERE id = $12
        RETURNING *
        "#,
        first_name,
        last_name,
        id_number,
        phone_number,
        address,
        city,
        state,
        zip_code,
        bank_name,
        bank_account_holder,
        bank_account_number,
        collector.id
    )
    .fetch_one(&app_state.pool)
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

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "collector": collector
        })),
    ))
}
