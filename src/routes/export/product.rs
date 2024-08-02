use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[utoipa::path(
    get,
    path = "/export/product",
    tag = "Export",
    security(("bearer_auth" = [])),
)]
pub async fn product(
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

    let products = sqlx::query!(
        r#"
            SELECT
                product.*,
                business.business_name AS business_name,
                business.phone_number AS business_phone_number,
                business_user.email AS business_email
            FROM public.product product
            LEFT JOIN business_profile business ON business.id = product.business_id
            LEFT JOIN users business_user ON business_user.id = business.user_id
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

    let mut csv_string: String =
        format!("\"Id\",\"Name\",\"Description\",\"Price (R)\",\"Business Name\",\"Business Phone Number\",\"Business Email\"");

    for product_record in products {
        csv_string += format!(
            "\n\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            product_record.id,
            product_record.name,
            product_record.description,
            product_record.price,
            product_record.business_name.unwrap_or("N/F".to_string()),
            product_record
                .business_phone_number
                .unwrap_or("N/F".to_string()),
            product_record.business_email.unwrap_or("N/F".to_string())
        )
        .as_str();
    }

    Ok((StatusCode::OK, csv_string))
}
