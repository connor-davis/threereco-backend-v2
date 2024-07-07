use axum::{http::StatusCode, Json};
use reqwest::Client;
use serde_json::{json, Value};

use crate::data::models::spanning::{
    spanning_backup::SpanningResponse, spanning_user::SpanningUser,
};

/// ## Spanning Backups Helper Function
///
/// Get the spanning backups authenticated with the spanning name and key from the
/// tenant.
///
/// ### Example
///
/// ```
/// use anyhow::Error;
/// use axum::http::StatusCode;
/// use crate::utils::functions::spanning_backups::get_spanning_backups;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let spanning_name = "whatevername";
///     let spanning_key = "whateverkey";
///     let tenant_name = "example";
///
///     let spanning_backups = get_spanning_backups(tenant_name, spanning_name, spanning_key).await.map_err(|error| {
///         tracing::error!("Error while getting spanning backups: {}", error);
///
///         Err(Error::msg("Error while getting spanning backups"))
///     })?
///
///     println!("{:?}", spanning_backups)
///
///     OK(())
/// }
/// ```
pub async fn get_spanning_backups(
    tenant_name: String,
    spanning_name: String,
    spanning_key: String,
) -> Result<Vec<SpanningUser>, (StatusCode, Json<Value>)> {
    tracing::info!("❕ Fetching Spanning Backups for {}", tenant_name);

    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .expect("Failed to create reqwest client.");

    let mut backups: Vec<SpanningUser> = Vec::new();
    let mut query_url = "https://o365-api-eu.spanningbackup.com/external/users".to_string();

    let mut response = client
        .get(query_url)
        .basic_auth(&spanning_name, Some(&spanning_key))
        .send()
        .await
        .map_err(|error| {
            tracing::error!("🔥 Error while fetching spanning backups: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
            )
        })?;

    let mut body = response.json::<SpanningResponse>().await.map_err(|error| {
        tracing::error!("🔥 Error while fetching spanning backups body: {}", error);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
        )
    })?;

    backups.extend(body.users);

    while body.next_link.is_some() {
        query_url = body.next_link.unwrap();

        if query_url.len() == 0 {
            tracing::info!("✅ Finished fetching Spanning Backups for {}", tenant_name);

            break;
        }

        tracing::info!(
            "❕ Fetching Spanning Backups for {} with {}",
            tenant_name,
            query_url
        );

        response = client
            .get(query_url)
            .basic_auth(&spanning_name, Some(&spanning_key))
            .send()
            .await
            .map_err(|error| {
                tracing::error!("🔥 Error while fetching spanning backups: {}", error);
    
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
                )
            })?;

        if response.status() != StatusCode::OK {
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(
                    json!({ "error": "Bad Gateway", "reason": "External api error occured. Please contact the api developer." }),
                ),
            ));
        }

        body = response.json::<SpanningResponse>().await.map_err(|error| {
            tracing::error!("🔥 Error while fetching spanning backups body: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer." }))
            )
        })?;

        backups.extend(body.users);

        if body.next_link.is_none() {
            tracing::info!("✅ Finished fetching Spanning Backups for {}", tenant_name);

            break;
        }
    }

    Ok(backups)
}
