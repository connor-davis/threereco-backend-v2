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
pub struct UpdateBusinessPayload {
    pub business_name: Option<String>,
    pub business_type: Option<String>,
    pub business_description: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
}

#[utoipa::path(
    post,
    path = "/business/{business_id}",
    params(("business_id" = String, Path, description = "The businesses id.")),
    request_body = UpdateBusinessPayload,
    tag = "Business",
    security(("bearer_auth" = [])),
)]
pub async fn business(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(business_id): extract::Path<Uuid>,
    extract::Json(payload): extract::Json<UpdateBusinessPayload>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a =
        authenticated_user.role() != Role::Staff && authenticated_user.role() != Role::SystemAdmin;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to update user data."
            })),
        ));
    }

    let existing_business = sqlx::query_as!(
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

    if existing_business.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "reason": "Business not found."
            })),
        ));
    }

    let business = existing_business.unwrap();

    let business_name = payload.business_name.unwrap_or(business.business_name);
    let business_type = payload.business_type.unwrap_or(business.business_type);
    let business_description = payload
        .business_description
        .unwrap_or(business.business_description);
    let phone_number = payload.phone_number.unwrap_or(business.phone_number);
    let address = payload.address.unwrap_or(business.address);
    let city = payload.city.unwrap_or(business.city);
    let state = payload.state.unwrap_or(business.state);
    let zip_code = payload.zip_code.unwrap_or(business.zip_code);

    let business = sqlx::query_as!(
        Business,
        r#"
        UPDATE business_profile
        SET business_name = $1, business_type = $2, business_description = $3, phone_number = $4, address = $5, city = $6, state = $7, zip_code = $8
        WHERE id = $9
        RETURNING *
        "#,
        business_name,
        business_type,
        business_description,
        phone_number,
        address,
        city,
        state,
        zip_code,
        business.id
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
            "business": business
        })),
    ))
}
