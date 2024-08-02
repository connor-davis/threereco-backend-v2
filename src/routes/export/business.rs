use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[utoipa::path(
    get,
    path = "/export/business",
    tag = "Export",
    security(("bearer_auth" = [])),
)]
pub async fn business(
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

    let businesses = sqlx::query!(
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
            FROM business_profile profile
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
        format!("\"Id\",\"Email\",\"Name\",\"Type\",\"Description\",\"Phone Number\",\"Address\",\"City\",\"Province\",\"Zip Code\"");

    for business_record in businesses {
        csv_string += format!(
            "\n\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            business_record.id,
            business_record.user_email,
            business_record.business_name,
            business_record.business_type,
            business_record
                .business_description,
            business_record.phone_number,
            business_record.address,
            business_record.city,
            business_record.state,
            business_record.zip_code
        )
        .as_str();
    }

    Ok((StatusCode::OK, csv_string))
}
