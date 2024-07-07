use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{collection::Collection, user::User},
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateCollectionPayload {
    pub business_id: Option<Uuid>,
    pub collector_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub weight: Option<BigDecimal>,
}

#[utoipa::path(
    post,
    path = "/collection/{collection_id}",
    params(("collection_id" = String, Path, description = "The collections id.")),
    request_body = UpdateCollectionPayload,
    tag = "Collection",
    security(("bearer_auth" = [])),
)]
pub async fn collection(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(collection_id): extract::Path<Uuid>,
    extract::Json(payload): extract::Json<UpdateCollectionPayload>,
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

    let existing_collection = sqlx::query_as!(
        Collection,
        r#"
        SELECT * FROM collection WHERE id = $1
        "#,
        collection_id
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

    if existing_collection.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "reason": "Collection not found."
            })),
        ));
    }

    let collection = existing_collection.unwrap();

    let business_id = payload.business_id.unwrap_or(collection.business_id);
    let collector_id = payload.collector_id.unwrap_or(collection.collector_id);
    let product_id = payload.product_id.unwrap_or(collection.product_id);
    let weight = payload.weight.unwrap_or(collection.weight);

    let collection = sqlx::query_as!(
        Collection,
        r#"
        UPDATE collection
        SET business_id = $1, collector_id = $2, product_id = $3, weight = $4
        WHERE id = $5
        RETURNING *
        "#,
        business_id,
        collector_id,
        product_id,
        weight,
        collection_id
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
            "collection": collection
        })),
    ))
}
