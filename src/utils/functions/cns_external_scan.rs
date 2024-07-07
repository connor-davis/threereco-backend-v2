#![allow(unused)]

use std::{fs::File, path::Path, time::Duration};

use anyhow::Error;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::{
    fs::{create_dir, try_exists},
    time::sleep,
};
use uuid::Uuid;

use crate::{data::models::customer::Customer, AppState};

/// ## CyberCNS External Scan Helper Function
///
/// Create a new external scan with CyberCNS which can be viewed later.
///
/// ### Example
///
/// ```
/// use anyhow::Error;
/// use axum::http::StatusCode;
/// use crate::utils::functions::cns_external_scan::create_external_scan;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let customer_id = "whateverid";
///     let username = "whatevername";
///     let password = "whateverkey";
///     let host_name = "http://example.com";
///
///     create_external_scan(
///         customer_id,
///         username,
///         password,
///         host_name
///     )
///         .await
///         .map_err(|error| {
///             tracing::error!("Error while creating new external scan: {}", error);
///
///             Err(Error::msg("Error while creating new external scan"))
///         })?
///
///     OK(())
/// }
/// ```
pub async fn create_external_scan(
    customer_id: Uuid,
    host_name: String,
    app_state: AppState,
) -> Result<(), Error> {
    let customer = match sqlx::query_as!(
        Customer,
        r#"
            UPDATE customers
            SET cns_scan_busy = $1
            WHERE id = $2
            RETURNING *;
        "#,
        true,
        customer_id
    )
    .fetch_optional(&app_state.db)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Error while finding customer: {}", error);

        error
    })? {
        Some(customer) => customer,
        None => return Err(Error::msg("Customer not found.")),
    };

    tracing::info!(
        "❕ Creating new CyberCNS external scan for {} for {}",
        customer.name,
        host_name
    );

    let customer_cns_id = match customer.cns_id {
        Some(cns_id) => Ok(cns_id),
        None => Err(Error::msg("Customer cns_id not found.")),
    }?;

    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .map_err(|error| {
            tracing::error!("🔥 Error while creating new reqwest client: {}", error);

            error
        })?;

    let request_body = json!({
        "hostname": host_name,
        "Scannow": true
    });

    let external_scan_response = client
        .post(format!(
            "https://portaleuwest2.mycybercns.com/api/company/{}/quickExternalScan",
            customer_cns_id,
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .header("Content-Type", "application/json")
        .basic_auth(
            &app_state.config.cyber_cns_client_id,
            Some(&app_state.config.cyber_cns_client_secret),
        )
        .json(&request_body)
        .send()
        .await
        .map_err(|error| {
            tracing::error!(
                "🔥 Error while creating new cybercns external scan: {}",
                error
            );

            error
        })?;

    let external_scan = external_scan_response
        .json::<(bool, String)>()
        .await
        .map_err(|error| {
            tracing::error!(
                "🔥 Error while creating new cybercns external scan: {}",
                error
            );

            error
        })?;

    if !external_scan.0 {
        return Err(Error::msg(
            "Failed to create quickExternalScan with CyberCNS.",
        ));
    }

    loop {
        sleep(Duration::from_secs(5)).await;

        let results = external_scan_results(
            customer.name.clone(),
            customer_cns_id.clone(),
            app_state.config.cyber_cns_client_id.clone(),
            app_state.config.cyber_cns_client_secret.clone(),
            host_name.clone(),
        )
        .await?;

        if !results.0 {
            continue;
        } else {
            // Create the reports directory if it doesn't exist.
            let directory_exists = try_exists("scans").await;

            match directory_exists {
                Ok(directory) => {
                    if directory {
                        tracing::info!("❕ \"scans\" directory found");
                    } else {
                        tracing::info!("❕ \"scans\" directory not found. creating");

                        let create_dir_result = create_dir("scans").await;

                        match create_dir_result {
                            Ok(_) => {
                                tracing::info!("✅ \"scans\" directory created");
                            }
                            Err(_) => {
                                tracing::info!("🔥 Failed to create \"scans\" directory");
                            }
                        }
                    }
                }
                Err(_) => {
                    tracing::info!("🔥 Unknown error when checking directory exists")
                }
            }

            let reports_dir = Path::new("scans");

            // Create the file name which includes the tenant name and the current date and time.
            let file_name = format!(
                "scan-{}-{}.json",
                customer_cns_id.clone(),
                host_name.clone(),
            );

            // Create the file path.
            let file_path = reports_dir.join(file_name.clone());

            // Create the file.
            let mut file = File::create(&file_path).expect("Failed to create the file.");

            // Write the data to the file.
            match serde_json::to_writer(
                &mut file,
                &json!({
                    "scan_host_name": host_name,
                    "scan_results": results.1
                }),
            ) {
                Ok(_) => tracing::info!("✅ Generated new scan results: {:?}", file_path),
                Err(error) => {
                    tracing::info!("🔥 External Scan Results Generation Error: {}", error)
                }
            }

            break;
        }
    }

    Ok(())
}

async fn external_scan_results(
    tenant_name: String,
    company_id: String,
    username: String,
    password: String,
    host_name: String,
) -> Result<(bool, Value), Error> {
    tracing::info!(
        "❕ Fetching CyberCNS external scan results for {} for {}",
        tenant_name,
        host_name
    );

    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .map_err(|error| {
            tracing::error!("🔥 Error while creating new reqwest client: {}", error);

            error
        })?;

    let request_body = json!({
        "hostname": host_name,
    });

    let external_scan_response = client
        .post(format!(
            "https://portaleuwest2.mycybercns.com/api/company/{}/quickExternalScanResults",
            company_id,
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .header("Content-Type", "application/json")
        .basic_auth(&username, Some(&password))
        .json(&request_body)
        .send()
        .await
        .map_err(|error| {
            tracing::error!(
                "🔥 Error while creating new cybercns external scan: {}",
                error
            );

            error
        })?;

    let external_scan_response = external_scan_response
        .json::<(bool, Value)>()
        .await
        .map_err(|error| {
            tracing::error!(
                "🔥 Error while creating new cybercns external scan: {}",
                error
            );

            error
        })?;

    Ok(external_scan_response)
}
