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
pub struct AddCollectionPayload {
    pub business_id: Uuid,
    pub collector_id: Uuid,
    pub product_id: Uuid,
    pub weight: BigDecimal,
}

#[utoipa::path(
    post,
    path = "/collection/add",
    request_body = AddCollectionPayload,
    tag = "Collection",
    security(("bearer_auth" = [])),
)]
pub async fn collection(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Json(payload): extract::Json<AddCollectionPayload>,
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

    let collection = sqlx::query_as!(
        Collection,
        r#"
            INSERT INTO collection (business_id, collector_id, product_id, weight)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#,
        payload.business_id,
        payload.collector_id,
        payload.product_id,
        payload.weight,
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
            "collection": collection,
        })),
    ))
}
