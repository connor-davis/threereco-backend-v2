use axum::{extract, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{collector::Collector, user::User},
    AppState,
};

#[utoipa::path(
    delete,
    path = "/collector/{collector_id}",
    params(("collector_id" = String, Path, description = "The collectors id.")),
    tag = "Collector",
    security(("bearer_auth" = [])),
)]
pub async fn collector(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(collector_id): extract::Path<Uuid>,
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
            Json(
                json!({ "error": "Internal Server Error", "reason": "Failed to query database." }),
            ),
        )
    })?;

    if existing_collector.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Not Found", "reason": "Collector not found." })),
        ));
    }

    let collector = existing_collector.unwrap();

    sqlx::query!(
        r#"
        DELETE FROM collector_profile WHERE id = $1
        "#,
        collector.id
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

    sqlx::query!(
        r#"
        DELETE FROM users WHERE id = $1
        "#,
        collector.user_id
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
