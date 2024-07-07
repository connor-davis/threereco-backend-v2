use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{business::Business, product::Product, user::User},
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AddProductPayload {
    pub business_id: Uuid,
    pub name: String,
    pub description: String,
    pub price: BigDecimal,
}

#[utoipa::path(
    post,
    path = "/product/add",
    request_body = AddProductPayload,
    tag = "Product",
    security(("bearer_auth" = [])),
)]
pub async fn product(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Json(payload): extract::Json<AddProductPayload>,
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

    let existing_business = sqlx::query_as!(
        Business,
        r#"
        SELECT * FROM business_profile WHERE id = $1
        "#,
        payload.business_id
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

    if existing_business.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Not Found", "reason": "Business not found." })),
        ));
    }

    let product = sqlx::query_as!(
        Product,
        r#"
        INSERT INTO product (business_id, name, description, price)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        payload.business_id,
        payload.name,
        payload.description,
        payload.price
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
            "product": product,
        })),
    ))
}
