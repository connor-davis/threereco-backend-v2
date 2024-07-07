use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsersQuery {
    pub role: Option<String>,
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    security(("bearer_auth" = [])),
)]
pub async fn users(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
    extract::Query(query): extract::Query<UsersQuery>,
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

    let users = sqlx::query_as!(
        User,
        r#"
            SELECT
                *
            FROM users
            WHERE
                id != $1
                AND role ILIKE '%' || $2 || '%'
            ORDER BY email ASC
        "#,
        authenticated_user.id,
        query.role
    )
        .fetch_all(&app_state.pool)
        .await
        .map_err(|error|{
            tracing::error!("🔥 Error while fetching all users: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "users": users
        })),
    ))
}

#[utoipa::path(
    get,
    path = "/users/{user_id}",
    params(("user_id" = String, Path, description = "The users id.")),
    tag = "Users",
    security(("bearer_auth" = [])),
)]
pub async fn user(
    extract::State(app_state): extract::State<AppState>,
    extract::Path(user_id): extract::Path<Uuid>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business;
    let requirement_b = user_id != authenticated_user.id;

    if requirement_a && requirement_b {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

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

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "user": user
        })),
    ))
}
