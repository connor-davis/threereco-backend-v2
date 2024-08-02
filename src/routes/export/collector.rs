use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[utoipa::path(
    get,
    path = "/export/collector",
    tag = "Export",
    security(("bearer_auth" = [])),
)]
pub async fn collector(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    if authenticated_user.role() != Role::Staff && authenticated_user.role() != Role::SystemAdmin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

    let collectors = sqlx::query!(
        r#"
            SELECT
                profile.*,
                u.email as user_email,
                u.password as user_password,
                u.role as user_role,
                u.active as user_active,
                u.mfa_enabled as user_mfa_enabled,
                u.mfa_verified as user_mfa_verified,
                u.mfa_secret as user_mfa_secret,
                u.created_at as user_created_at,
                u.updated_at as user_updated_at
            FROM collector_profile profile
            LEFT JOIN users u ON profile.user_id = u.id
        "#
    )
    .fetch_all(&app_state.pool)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Error while querying businesses: {}", error);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Internal Server Error",
                "reason": "Unknown error occured. Please contact the api developer."
            })),
        )
    })?;

    let mut csv_string =
        format!("\"Id\",\"Email\",\"First Name\",\"Last Name\",\"ID Number\",\"Phone Number\",\"Address\",\"City\",\"Province\",\"Zip Code\",\"Bank Name\",\"Bank Account Holder\",\"Bank Account Number\"");

    for collector_record in collectors {
        csv_string += format!(
            "\n\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            collector_record.id,
            collector_record.user_email.unwrap_or("-".to_string()),
            collector_record.first_name,
            collector_record.last_name,
            collector_record.id_number,
            collector_record.phone_number,
            collector_record.address,
            collector_record.city,
            collector_record.state,
            collector_record.zip_code,
            collector_record.bank_name,
            collector_record.bank_account_holder,
            collector_record.bank_account_number,
        )
        .as_str();
    }

    Ok((StatusCode::OK, csv_string))
}
