use axum::{extract, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[utoipa::path(
    delete,
    path = "/users/{user_id}",
    params(("user_id" = String, Path, description = "The users id.")),
    tag = "Users",
    security(("bearer_auth" = [])),
)]
pub async fn user(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(user_id): extract::Path<Uuid>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let user = match sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE id = $1 AND id != $2"#,
        user_id,
        authenticated_user.id
    )
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|error| {
            tracing::error!("🔥 Error while fetching user: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
            )
        })? {
            Some(user) => Ok(user),
            None => Err((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Not Found", "reason": "User not found." })),
            ))
        }?;

    if user.role() == Role::SystemAdmin && authenticated_user.role() != Role::SystemAdmin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to delete \"System Admin\" users."
            })),
        ));
    }

    if user.role() != Role::Collector && user.role() == Role::Business {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You only have permission to delete \"Collector\" users."
            })),
        ));
    }

    sqlx::query!(
        r#"DELETE FROM users WHERE id = $1"#,
        user_id
    )
        .execute(&app_state.pool)
        .await
        .map_err(|error|{
            tracing::error!("🔥 Error occured while deleting user: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true
        })),
    ))
}
