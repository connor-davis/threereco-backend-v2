use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use serde_json::{json, Value};

use crate::{authentication::roles::Role, data::entities::user::User, AppState};

#[utoipa::path(
    get,
    path = "/export/collection",
    tag = "Export",
    security(("bearer_auth" = [])),
)]
pub async fn collection(
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

    let collections = sqlx::query!(
        r#"
            SELECT 
                collection.id AS id,
                collection.weight AS weight,
                product.price AS price,
                product.price * collection.weight AS total_price,
                business.business_name AS business_name,
                business.phone_number AS business_phone_number,
                CONCAT(business.address, ', ', business.city, ', ', business.state, ', ', business.zip_code) AS business_location,
                business_user.email AS business_email,
                CONCAT(collector.first_name, ' ', collector.last_name) AS collector_full_name,
                collector.id_number AS collector_id_number,
                collector.phone_number AS collector_phone_number,
                CONCAT(collector.address, ', ', collector.city, ', ', collector.state, ', ', collector.zip_code) AS collector_location,
                collector.bank_name AS collector_bank_name,
                collector.bank_account_holder AS collector_bank_account_holder,
                collector.bank_account_number AS collector_bank_account_number,
	            collector_user.email AS collector_email
            FROM public.collection collection
            LEFT JOIN public.business_profile business ON business.id = collection.business_id
            LEFT JOIN public.users business_user ON business_user.id = business.user_id
            LEFT JOIN public.collector_profile collector ON collector.id = collection.collector_id
            LEFT JOIN public.users collector_user ON collector_user.id = collector.user_id
            LEFT JOIN public.product product ON product.id = collection.product_id
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
        format!("\"Id\",\"Collection Weight (kg)\",\"Product Price (R)\",\"Collection Total Price (R)\",\"Business Name\",\"Business Phone Number\",\"Business Location\",\"Business Email\",\"Collector Full Name\",\"Collector ID Number\",\"Collector Phone Number\",\"Collector Location\",\"Collector Bank Name\",\"Collector Bank Account Holder\",\"Collector Bank Account Number\",\"Collector Email\"");

    for collection_record in collections {
        csv_string += format!(
            "\n\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            collection_record.id,
            collection_record.weight,
            collection_record.price,
            collection_record.total_price,
            collection_record.business_name,
            collection_record.business_phone_number,
            collection_record.business_location,
            collection_record.business_email,
            collection_record.collector_full_name,
            collection_record.collector_id_number,
            collection_record.collector_phone_number,
            collection_record.collector_location,
            collection_record.collector_bank_name,
            collection_record.collector_bank_account_holder,
            collection_record.collector_bank_account_number,
            collection_record.collector_email,
        )
        .as_str();
    }

    Ok((StatusCode::OK, csv_string))
}
