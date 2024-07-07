use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{product::Product, user::User},
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateProductPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<BigDecimal>,
}

#[utoipa::path(
    post,
    path = "/product/{product_id}",
    params(("product_id" = String, Path, description = "The products id.")),
    request_body = UpdateProductPayload,
    tag = "Product",
    security(("bearer_auth" = [])),
)]
pub async fn product(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(product_id): extract::Path<Uuid>,
    extract::Json(payload): extract::Json<UpdateProductPayload>,
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

    let existing_product = sqlx::query_as!(
        Product,
        r#"
        SELECT * FROM product WHERE id = $1
        "#,
        product_id
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

    if existing_product.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "reason": "Product not found."
            })),
        ));
    }

    let product = existing_product.unwrap();

    let name = payload.name.unwrap_or(product.name);
    let description = payload.description.unwrap_or(product.description);
    let price = payload.price.unwrap_or(product.price);

    let product = sqlx::query_as!(
        Product,
        r#"
        UPDATE product
        SET name = $1, description = $2, price = $3
        WHERE id = $4
        RETURNING *
        "#,
        name,
        description,
        price,
        product.id
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
            "product": product
        })),
    ))
}
