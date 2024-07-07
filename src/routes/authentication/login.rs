use axum::{extract, response::IntoResponse, Json};
use bcrypt::verify;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::{
    authentication::jwt::{create_jwt, Claims},
    data::entities::user::User,
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LoginPayload {
    #[schema(example = "")]
    pub email: String,
    #[schema(example = "")]
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/authentication/login",
    tag = "Authentication",
    request_body = LoginPayload,
    responses(
        (
            status = 200,
            content_type = "application/json",
            description = "Authorized.",
        ),
        (
            status = 401,
            content_type = "application/json",
            description = "Unauthorized.",
        ),
        (
            status = 500,
            content_type = "application/json",
            description = "Internal Server Error. Please contact the developer.",
        ),
    ),
)]
pub async fn user(
    extract::State(app_state): extract::State<AppState>,
    extract::Json(payload): extract::Json<LoginPayload>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let email = payload.email;
    let password = payload.password;

    // Update the users MFA status if it is in any form positive.
    sqlx::query!(r#"
            UPDATE users
            SET mfa_verified = $1
            WHERE email = $2
        "#,
        false,
        email
    )
        .execute(&app_state.pool)
        .await
        .map_err(|error|{
            tracing::error!("🔥 Failed to update user MFA verification status: {}", error);

            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal Server Error", "reason": "Unknown error occured. Please contact the api developer."})))
        })?;

    // Find the user with the email.
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT * FROM users
            WHERE email = $1
        "#,
        email
    )
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|error|{
            tracing::error!("🔥 Failed to query user in the database: {}", error);

            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"Internal Server Error","reason":"Unknown error occured. Please contact the api developer."})))
        })?
        .ok_or_else(||{
            (StatusCode::NOT_FOUND,Json(json!({"error": "Unauthorized","reason":"User does not exist."})))
        })?;

    // Check if the users account is active.
    if !user.active {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized","reason": "Account deactivated."})),
        ));
    }

    // Verify the users password.
    let password_matches = verify(password, &user.password).map_err(|error|{
        tracing::error!("🔥 Failed to verify user password: {}", error);

        (StatusCode::INTERNAL_SERVER_ERROR,Json(json!({"error":"Internal Server Error","reason":"Unkown error occured. Please contact the api developer."})))
    })?;

    if !password_matches {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"Unauthorized","reason":"Invalid password."})),
        ));
    }

    // Generate token claims.
    let claims = Claims {
        sub: user.clone().email,
        iss: "Thusa Managed Executive Reports API.".to_string(),
        role: user.role.to_string(),
        ..Default::default()
    };

    // Generate token.
    let token = create_jwt(claims).await.map_err(|error|{
        tracing::error!("🔥 Failed to create JWT token: {}", error);

        (StatusCode::INTERNAL_SERVER_ERROR,Json(json!({"error":"Internal Server Error","reason":"Unknown error occured. Please contact the api developer."})))
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "user": user,
            "token": token
        })),
    ))
}
