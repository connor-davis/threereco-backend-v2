use axum::{extract, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{product::Product, user::User},
    AppState,
};

#[utoipa::path(
    delete,
    path = "/product/{product_id}",
    params(("product_id" = String, Path, description = "The products id.")),
    tag = "Product",
    security(("bearer_auth" = [])),
)]
pub async fn product(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(product_id): extract::Path<Uuid>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(
                json!({ "error": "Unauthorized", "reason": "You do not have permission to delete user data." }),
            ),
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
            Json(
                json!({ "error": "Internal Server Error", "reason": "Failed to query database." }),
            ),
        )
    })?;

    if existing_product.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Not Found", "reason": "Product not found." })),
        ));
    }

    let product = existing_product.unwrap();

    sqlx::query!(
        r#"
        DELETE FROM product WHERE id = $1
        "#,
        product.id
    )
    .execute(&app_state.pool)
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
            "success": true
        })),
    ))
}
