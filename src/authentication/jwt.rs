use std::env;

use anyhow::Error;
use axum::{
    extract::{self, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{data::entities::user::User, AppState};

use super::roles::Role;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

impl Default for Claims {
    fn default() -> Self {
        let mut now = Utc::now();
        let iat = now.timestamp() as usize;
        let exp = Duration::try_days(1).unwrap();
        now += exp;
        let exp = now.timestamp() as usize;
        let role = Role::Collector.to_string();

        Self {
            sub: "".to_string(),
            iss: "".to_string(),
            exp,
            iat,
            role,
        }
    }
}

#[allow(unused)]
pub async fn create_jwt(claims: Claims) -> Result<String, Error> {
    let jwt_secret = env::var("JWT_SECRET").map_err(|error| {
        tracing::error!("🔥 Failed to load JWT_SECRET from .env file: {}", error);
        error
    })?;

    let key = EncodingKey::from_secret(jwt_secret.as_bytes());

    let token = encode(&Header::default(), &claims, &key).map_err(|error| {
        tracing::error!("🔥 Failed to create JWT: {}", error);
        error
    })?;

    Ok(token)
}

#[allow(unused)]
pub async fn jwt_guard(
    extract::State(app_state): extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let token = request
        .headers()
        .get("Authorization")
        .map(|header| {
            let header = header.to_str().unwrap_or_else(|error| {
                tracing::error!("🔥 Failed to parse Authorization header: {}", error);
                ""
            });

            header
        })
        .unwrap_or_else(|| {
            tracing::error!("🔥 Failed to get Authorization header.");
            ""
        })
        .replace("Bearer ", "");

    let claims = validate_jwt(&token, app_state.clone()).await?;
    let email = &claims.sub;

    let user = sqlx::query_as!(
        User,
        r#"
            SELECT * FROM users WHERE email = $1
        "#,
        email
    )
    .fetch_one(&app_state.pool)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Failed to get user: {}", error);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Internal Server Error" })),
        )
    })?;

    request.extensions_mut().insert::<User>(user);

    Ok(next.run(request).await)
}

#[allow(unused)]
pub async fn jwt_guard_optional(
    extract::State(app_state): extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let token = request.headers().get("Authorization").map(|header| {
        let header = header.to_str().unwrap_or_else(|error| {
            tracing::error!("🔥 Failed to parse Authorization header: {}", error);
            ""
        });

        header
    });

    match token {
        Some(token) => {
            let token = token.replace("Bearer ", "");

            let claims = validate_jwt(&token, app_state.clone()).await?;
            let email = &claims.sub;

            let user = sqlx::query_as!(
                User,
                r#"
                    SELECT * FROM users WHERE email = $1
                "#,
                email
            )
            .fetch_one(&app_state.pool)
            .await
            .map_err(|error| {
                tracing::error!("🔥 Failed to get user: {}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Internal Server Error" })),
                )
            })?;

            request.extensions_mut().insert::<Option<User>>(Some(user));

            Ok(next.run(request).await)
        }
        None => Ok(next.run(request).await),
    }
}

pub async fn validate_jwt(
    token: &str,
    app_state: AppState,
) -> Result<Claims, (StatusCode, Json<Value>)> {
    let jwt_secret = app_state.config.jwt_secret;

    let key = DecodingKey::from_secret(jwt_secret.as_bytes());

    let jwt_claims = decode::<Claims>(token, &key, &Validation::default()).map_err(|error| match error.kind() {
        ErrorKind::ExpiredSignature => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Unauthorized", "reason": "Token expired." })),
        ),
        ErrorKind::InvalidToken => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Unauthorized", "reason": "Invalid token." })),
        ),
        _ => {
            tracing::error!("🔥 Failed to validate JWT: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Internal server error. Please contact the developer." })),
            )
        }
    })?;

    let email = &jwt_claims.claims.sub;
    let role = &jwt_claims.claims.role;

    tracing::info!("🔐 User {} authenticated with role {}.", email, role);

    Ok(jwt_claims.claims)
}
