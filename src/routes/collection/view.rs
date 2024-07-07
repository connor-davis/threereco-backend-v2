use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{
        business::Business, collection::Collection, collector::Collector, user::User,
    },
    AppState,
};

#[utoipa::path(
    get,
    path = "/collection",
    tag = "Collection",
    security(("bearer_auth" = [])),
)]
pub async fn collections(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    if authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

    match authenticated_user.role() {
        Role::Collector => {
            let collector = sqlx::query_as!(
                Collector,
                r#"
                SELECT * FROM collector_profile WHERE user_id = $1
                "#,
                authenticated_user.id
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

            let collections = sqlx::query_as!(
                Collection,
                r#"
                SELECT * FROM collection WHERE collector_id = $1
                "#,
                collector.id
            )
            .fetch_all(&app_state.pool)
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
                    "collections": collections
                })),
            ))
        }
        Role::Business => {
            let business = sqlx::query_as!(
                Business,
                r#"
                SELECT * FROM business_profile WHERE user_id = $1
                "#,
                authenticated_user.id
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

            let collections = sqlx::query_as!(
                Collection,
                r#"
                SELECT * FROM collection WHERE business_id = $1
                "#,
                business.id
            )
            .fetch_all(&app_state.pool)
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
                    "collections": collections
                })),
            ))
        }
        _ => {
            let collections = sqlx::query_as!(
                Collection,
                r#"
                SELECT * FROM collection
                "#
            )
            .fetch_all(&app_state.pool)
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
                    "collections": collections
                })),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/collection/{collection_id}",
    params(("collection_id" = String, Path, description = "The collections id.")),
    tag = "Collection",
    security(("bearer_auth" = [])),
)]
pub async fn collection(
    extract::State(app_state): extract::State<AppState>,
    extract::Path(collection_id): extract::Path<Uuid>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

    let collection = sqlx::query_as!(
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

    if collection.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "reason": "Product not found."
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "collection": collection.unwrap()
        })),
    ))
}
