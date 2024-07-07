use axum::{extract, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{collection::Collection, user::User},
    AppState,
};

#[utoipa::path(
    delete,
    path = "/collection/{collection_id}",
    params(("collection_id" = String, Path, description = "The collections id.")),
    tag = "Collection",
    security(("bearer_auth" = [])),
)]
pub async fn collection(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(collection_id): extract::Path<Uuid>,
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
            Json(
                json!({ "error": "Internal Server Error", "reason": "Failed to query database." }),
            ),
        )
    })?;

    if existing_collection.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Not Found", "reason": "Collection not found." })),
        ));
    }

    let collection = existing_collection.unwrap();

    sqlx::query!(
        r#"
        DELETE FROM collection WHERE id = $1
        "#,
        collection.id
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
