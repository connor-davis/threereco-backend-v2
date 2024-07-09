use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchCollector {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub id_number: String,
    pub phone_number: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub bank_name: String,
    pub bank_account_holder: String,
    pub bank_account_number: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub email: Option<String>,
}

#[utoipa::path(
    get,
    path = "/collector/search/{query}",
    params(("query" = String, Path, description = "The search query.")),
    tag = "Collector",
    security(("bearer_auth" = [])),
)]
pub async fn collector(
    extract::State(app_state): extract::State<AppState>,
    extract::Path(query): extract::Path<String>,
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

    let collectors = sqlx::query_as!(
        SearchCollector,
        r#"
        SELECT
            collector_profile.*,
            users.email AS email
        FROM
        collector_profile
        LEFT JOIN users ON collector_profile.user_id = users.id
        WHERE
            id_number ILIKE '%' || $1 || '%'
            OR first_name ILIKE '%' || $1 || '%'
            OR last_name ILIKE '%' || $1 || '%'
            OR phone_number ILIKE '%' || $1 || '%'
            OR users.email ILIKE '%' || $1 || '%'
        "#,
        query
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
            "collectors": collectors
        })),
    ))
}
