use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use rand::Rng;
use serde_json::{json, Value};
use totp_rs::{Algorithm, TOTP};

use crate::{data::entities::user::User, AppState};

/// Generate a random string
///
/// ### Example
/// ```rust
/// use crate::routes::mfa::generate::generate_random_string;
///
/// let random_string = generate_random_string();
/// ```
fn generate_random_string() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    let hex_string: String = random_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    hex_string
}

pub async fn generate(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let secret_string = generate_random_string();
    let secret = secret_string.as_bytes().to_vec();

    let totp = TOTP::new(
        Algorithm::SHA256,
        6,
        1,
        30,
        secret,
        Some(app_state.config.mfa_issuer.to_string()),
        authenticated_user.email.to_string(),
    );

    match totp {
        Ok(totp) => {
            sqlx::query!(
                r#"
                    UPDATE users
                    SET mfa_secret = $1
                    WHERE email = $2
                "#,
                secret_string,
                authenticated_user.email
            )
            .execute(&app_state.pool)
            .await
            .map_err(|error| {
                tracing::error!("🔥 Failed to update MFA secret: {}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to update MFA secret" })),
                )
            })?;

            let qr_code = totp.get_qr_base64().unwrap();
            let qr_code = format!("data:image/png;base64,{}", qr_code);

            Ok((StatusCode::OK, qr_code))
        }
        Err(error) => {
            tracing::error!("🔥 Failed to generate TOTP: {}", error);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to generate TOTP" })),
            ))
        }
    }
}
