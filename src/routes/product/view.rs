use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    authentication::roles::Role,
    data::entities::{business::Business, product::Product, user::User},
    AppState,
};

#[utoipa::path(
    get,
    path = "/product",
    tag = "Product",
    security(("bearer_auth" = [])),
)]
pub async fn products(
    extract::State(app_state): extract::State<AppState>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    if authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

    match authenticated_user.role() {
        Role::Business => {
            let business = sqlx::query_as!(
                Business,
                r#"
                SELECT * FROM business_profile WHERE user_id = $1
                "#,
                authenticated_user.id
            )
            .fetch_one(&app_state.pool)
            .await
            .map_err(|error| {
                tracing::error!("🔥 Failed to query database: {}", error);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal Server Error",
                        "reason": "Failed to query database."
                    })),
                )
            })?;

            let products = sqlx::query_as!(
                Product,
                r#"
                SELECT * FROM product WHERE business_id = $1
                "#,
                business.id
            )
            .fetch_all(&app_state.pool)
            .await
            .map_err(|error| {
                tracing::error!("🔥 Failed to query database: {}", error);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal Server Error",
                        "reason": "Failed to query database."
                    })),
                )
            })?;

            Ok((
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "products": products
                })),
            ))
        }
        _ => {
            let products = sqlx::query_as!(
                Product,
                r#"
                SELECT * FROM product
                "#
            )
            .fetch_all(&app_state.pool)
            .await
            .map_err(|error| {
                tracing::error!("🔥 Failed to query database: {}", error);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal Server Error",
                        "reason": "Failed to query database."
                    })),
                )
            })?;

            Ok((
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "products": products
                })),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/product/{product_id}",
    params(("product_id" = String, Path, description = "The products id.")),
    tag = "Product",
    security(("bearer_auth" = [])),
)]
pub async fn product(
    extract::State(app_state): extract::State<AppState>,
    extract::Path(product_id): extract::Path<Uuid>,
    extract::Extension(authenticated_user): extract::Extension<User>,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, Json<Value>)> {
    let requirement_a = authenticated_user.role() != Role::Staff
        && authenticated_user.role() != Role::SystemAdmin
        && authenticated_user.role() != Role::Business;

    if requirement_a {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "reason": "You do not have permission to access user data."
            })),
        ));
    }

    let product = sqlx::query_as!(
        Product,
        r#"
        SELECT * FROM product WHERE id = $1
        "#,
        product_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|error| {
        tracing::error!("🔥 Failed to query database: {}", error);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Internal Server Error",
                "reason": "Failed to query database."
            })),
        )
    })?;

    if product.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "reason": "Product not found."
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "product": product.unwrap()
        })),
    ))
}
