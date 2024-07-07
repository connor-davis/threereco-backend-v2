use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserPayload {
    pub email: Option<String>,
    pub role: Option<String>,
}

#[utoipa::path(
    post,
    path = "/users/{user_id}",
    params(("user_id" = String, Path, description = "The users id.")),
    request_body = UpdateUserPayload,
    tag = "Users",
    security(("bearer_auth" = [])),
)]
pub async fn user(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Path(user_id): extract::Path<Uuid>,
    extract::Json(payload): extract::Json<UpdateUserPayload>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    if payload.role.is_some() {
        let role = payload.role.clone().unwrap();

        if role == Role::SystemAdmin.to_string() && authenticated_user.role() != Role::SystemAdmin {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Unauthorized",
                    "reason": "You do not have permission to update a users role to \"System Admin\"."
                })),
            ));
        }
    }

    let user = match sqlx::query_as!(
        User,
        r#"
            SELECT
                *
            FROM users
            WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&app_state.pool)
    .await
    {
        Ok(user) => Ok(user),
        Err(error) => {
            tracing::error!("🔥 Error while finding existing user: {}", error);

            Err((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Not Found", "reason": "User not found." })),
            ))
        }
    }?;

    if user.role() == Role::SystemAdmin && authenticated_user.role() != Role::SystemAdmin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to remove \"System Admin\" role from a user."
            })),
        ));
    }

    if user.role() != Role::Collector && user.role() == Role::Business {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You only have permission to update \"Collector\" users."
            })),
        ));
    }

    let user = sqlx::query_as!(
        User,
        r#"
            UPDATE users
            SET
                email = $1,
                role = $2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            RETURNING *
        "#,
        payload.email.or(Some(user.clone().email)),
        payload.role.or(Some(user.clone().role().to_string())),
        user_id
    )
        .fetch_one(&app_state.pool)
        .await
        .map_err(|error| {
            tracing::error!("🔥 Error while updating user: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "user": user,
        })),
    ))
}
