use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use totp_rs::{Algorithm, TOTP};

use crate::{data::entities::user::User, AppState};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyMFAPayload {
    pub code: String,
}

pub async fn verify(
    extract::Query(params): extract::Query<VerifyMFAPayload>,
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let user_secret_string = authenticated_user.mfa_secret.clone().ok_or_else(|| {
        tracing::error!("🔥 MFA secret not found.");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "MFA secret not found" })),
        )
    })?;

    let user_secret = user_secret_string.clone().into_bytes().to_vec();

    let totp = TOTP::new(
        Algorithm::SHA256,
        6,
        1,
        30,
        user_secret,
        Some(app_state.config.mfa_issuer.to_string()),
        authenticated_user.email.to_string(),
    );

    match totp {
        Ok(totp) => {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let token = totp.generate(time);

            if params.code.trim() == token {
                sqlx::query!(
                    r#"
                        UPDATE users
                        SET mfa_verified = $1
                        WHERE id = $2
                    "#,
                    true,
                    authenticated_user.id
                )
                .execute(&app_state.pool)
                .await
                .map_err(|error| {
                    println!("🔥 Failed to update MFA verification: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to update MFA verification" })),
                    )
                })?;

                sqlx::query!(
                    r#"
                        UPDATE users
                        SET mfa_enabled = $1
                        WHERE id = $2
                    "#,
                    true,
                    authenticated_user.id
                )
                .execute(&app_state.pool)
                .await
                .map_err(|error| {
                    println!("🔥 Failed to update MFA verification: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to update MFA verification" })),
                    )
                })?;

                let user = sqlx::query_as!(
                    User,
                    r#"
                        SELECT *
                        FROM users
                        WHERE id = $1
                    "#,
                    authenticated_user.id
                )
                .fetch_one(&app_state.pool)
                .await
                .map_err(|error| {
                    tracing::error!("🔥 Failed to query database: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Internal server error. Please contact the developer." })),
                    )
                })?;

                Ok((StatusCode::OK, Json(user)))
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(
                        json!({ "error": "Unauthorized", "reason": "Failed to verify MFA token." }),
                    ),
                ))
            }
        }
        Err(error) => {
            tracing::error!("🔥 Failed to generate TOTP: {}", error);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    json!({ "error": "Internal Server Error", "reason": "Please contact the developer." }),
                ),
            ))
        }
    }
}
