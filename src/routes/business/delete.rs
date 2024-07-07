use axum::{extract, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{business::Business, user::User},
    AppState,
};

#[utoipa::path(
    delete,
    path = "/business/{business_id}",
    params(("business_id" = String, Path, description = "The businesses id.")),
    tag = "Business",
    security(("bearer_auth" = [])),
)]
pub async fn business(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(business_id): extract::Path<Uuid>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::SystemAdmin;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(
                json!({ "error": "Unauthorized", "reason": "You do not have permission to delete user data." }),
            ),
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

    let business = existing_business.unwrap();

    sqlx::query!(
        r#"
        DELETE FROM business_profile WHERE id = $1
        "#,
        business.id
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
        business.user_id
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
