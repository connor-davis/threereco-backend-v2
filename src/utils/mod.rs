use anyhow::Error;
use chrono::{DateTime, Days, Utc};
use serde_json::{json, Value};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::data::{models::customer::Customer, reports::Reports};

pub mod access_token;
pub mod functions;

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

pub async fn generate_report(
    id: Uuid,
    customer_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    pool: Pool<Postgres>,
) -> Result<(), Error> {
    let handler = Reports::init().await;

    let customer = sqlx::query_as!(
        Customer,
        r#"
        SELECT *
        FROM customers
        WHERE id = $1
    "#,
        customer_id
    )
    .fetch_optional(&pool)
    .await?;

    if customer.is_none() {
        return Err(Error::msg("Failed to find customer for report."));
    }

    let customer = customer.unwrap();

    tracing::info!("❕ Generating Report.");

    match customer.vsa_id {
        Some(vsa_org_id) => {
            let report = match handler.read_report_file(id).await? {
                Some(mut report) => {
                    merge_json(
                        &mut report,
                        &json!({
                            "report_status": "Generating VSA data."
                        }),
                    );

                    report
                }
                None => json!({}),
            };

            handler.write_report_file(id, report).await?;

            handler.generate_vsa_data(vsa_org_id, id).await?
        }
        None => {}
    };

    match customer.cns_id {
        Some(cns_company_id) => {
            let report = match handler.read_report_file(id).await? {
                Some(mut report) => {
                    merge_json(
                        &mut report,
                        &json!({
                            "report_status": "Generating CyberCNS data."
                        }),
                    );

                    report
                }
                None => json!({}),
            };

            handler.write_report_file(id, report).await?;

            handler
                .generate_cns_data(cns_company_id, id, start_date, end_date)
                .await?
        }
        None => {}
    };

    match customer.rocket_id {
        Some(rocket_account_id) => {
            let report = match handler.read_report_file(id).await? {
                Some(mut report) => {
                    merge_json(
                        &mut report,
                        &json!({
                            "report_status": "Generating Rocket Cyber data."
                        }),
                    );

                    report
                }
                None => json!({}),
            };

            handler.write_report_file(id, report).await?;

            handler
                .generate_rocket_data(rocket_account_id, id, start_date, end_date)
                .await?
        }
        None => {}
    };

    match customer.spanning_user {
        Some(spanning_user) => match customer.spanning_key {
            Some(spanning_key) => match end_date.checked_sub_days(Days::new(7)) {
                Some(start_date) => {
                    let report = match handler.read_report_file(id).await? {
                        Some(mut report) => {
                            merge_json(
                                &mut report,
                                &json!({
                                    "report_status": "Generating Spanning Backups data."
                                }),
                            );

                            report
                        }
                        None => json!({}),
                    };

                    handler.write_report_file(id, report).await?;

                    handler
                        .generate_spanning_data(
                            spanning_user,
                            spanning_key,
                            id,
                            start_date.timestamp_millis(),
                            end_date.timestamp_millis(),
                        )
                        .await
                        .map_err(|error| {
                            tracing::error!(
                                "🔥 Error while generating Spanning Backups data: {}",
                                error
                            );

                            error
                        })?
                }
                None => {
                    tracing::error!("🔥 Failed to create 7 days before end date for report for Spanning Backups data generation.");
                }
            },
            None => {}
        },
        None => {}
    };

    match customer.veeam_id {
        Some(veeam_company_id) => {
            let report = match handler.read_report_file(id).await? {
                Some(mut report) => {
                    merge_json(
                        &mut report,
                        &json!({
                            "report_status": "Generating Veeam Backups data."
                        }),
                    );

                    report
                }
                None => json!({}),
            };

            handler.write_report_file(id, report).await?;

            handler.generate_veeam_data(veeam_company_id, id).await
            .map_err(|error| {
                tracing::error!(
                    "🔥 Error while generating Veeam Backups data: {}",
                    error
                );

                error
            })?
        }
        None => {}
    };

    tracing::info!("✅ Generated Report.");

    let report = match handler.read_report_file(id).await? {
        Some(mut report) => {
            merge_json(
                &mut report,
                &json!({
                    "report_status": "Ready."
                }),
            );

            report
        }
        None => json!({}),
    };

    handler.write_report_file(id, report).await?;

    Ok(())
}
