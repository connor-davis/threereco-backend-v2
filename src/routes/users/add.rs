use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use bcrypt::hash;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AddUserPayload {
    pub email: String,
    pub password: String,
    pub role: Role,
}

#[utoipa::path(
    post,
    path = "/users/add",
    request_body = AddUserPayload,
    tag = "Users",
    security(("bearer_auth" = [])),
)]
pub async fn user(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Json(payload): extract::Json<AddUserPayload>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business;
    let requirement_b =
        payload.role == Role::SystemAdmin && authenticated_user.role() != Role::SystemAdmin;
    let requirement_c =
        payload.role != Role::Collector && authenticated_user.role() == Role::Business;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(
                json!({ "error": "Unauthorized", "reason": "You do not have permission to add users." }),
            ),
        ));
    }

    if requirement_b {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(
                json!({ "error": "Unauthorized", "reason": "You do not have permission to add users with role \"System Admin\"." }),
            ),
        ));
    }

    if requirement_c {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(
                json!({ "error": "Unauthorized", "reason": "You only have permission to add users with role \"Collector\"." }),
            ),
        ));
    }

    // Find an existing user.
    let existing_user = sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE TRIM(LOWER(email)) = TRIM(LOWER($1))"#,
        payload.email
    )
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|error|{
            tracing::error!("🔥 Error while finding existing user: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer."}))
            )
        })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(
                json!({ "error":"Conflict", "reason": "A user with that email already exists. Please try again with a different email." }),
            ),
        ));
    }

    // Hash new user object password.
    let hashed_password = hash(payload.password, 4).map_err(|error| {
        tracing::error!("🔥 Failed to hash new user password: {}", error);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." })),
        )
    })?;

    // Create new user object and retrieve uuid
    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO users (email, password, role) VALUES ($1,$2,$3) RETURNING *"#,
        payload.email,
        hashed_password,
        payload.role.to_string()
    )
        .fetch_one(&app_state.pool)
        .await
        .map_err(|error|{
            tracing::error!("🔥 Error while creating new user: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal Server Error","reason":"Unknown error occured. Please contact the api developer."}))
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
