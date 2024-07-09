use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    authentication::jwt,
    documentation::api_documentation::ApiDoc,
    routes::{
        authentication, business, collection, collector, fallback::get_fallback, index::get_index,
        mfa, product, users,
    },
    AppState,
};

pub async fn create_router(app_state: AppState) -> Router {
    Router::new()
        // business routes
        .nest(
            "/business",
            Router::new()
                .route("/", get(business::view::businesses))
                .route(
                    "/:business_id",
                    get(business::view::business)
                        .post(business::update::business)
                        .delete(business::delete::business),
                )
                .route("/add", post(business::add::business)),
        )
        .nest(
            "/collector",
            Router::new()
                .route("/", get(collector::view::collectors))
                .route(
                    "/:collector_id",
                    get(collector::view::collector)
                        .post(collector::update::collector)
                        .delete(collector::delete::collector),
                )
                .route("/search/:query", get(collector::search::collector))
                .route("/add", post(collector::add::collector)),
        )
        .nest(
            "/product",
            Router::new()
                .route("/", get(product::view::products))
                .route(
                    "/:product_id",
                    get(product::view::product)
                        .post(product::update::product)
                        .delete(product::delete::product),
                )
                .route("/add", post(product::add::product)),
        )
        .nest(
            "/collection",
            Router::new()
                .route("/", get(collection::view::collections))
                .route(
                    "/:collection_id",
                    get(collection::view::collection)
                        .post(collection::update::collection)
                        .delete(collection::delete::collection),
                )
                .route("/add", post(collection::add::collection)),
        )
        .nest(
            "/users",
            Router::new()
                .route("/", get(users::view::users))
                .route(
                    "/:user_id",
                    get(users::view::user)
                        .post(users::update::user)
                        .delete(users::delete::user),
                )
                .route("/add", post(users::add::user)),
        )
        .nest(
            "/authentication",
            Router::new()
                .route("/check", get(authentication::check::index))
                .nest(
                    "/mfa",
                    Router::new()
                        .route("/generate", get(mfa::generate::generate))
                        .route("/verify", post(mfa::verify::verify)),
                ),
        )
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            jwt::jwt_guard,
        ))
        // authentication
        .nest(
            "/authentication",
            Router::new().route("/login", post(authentication::login::user)),
        )
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            jwt::jwt_guard_optional,
        ))
        // default routes
        .route("/", get(get_index))
        // logs
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .fallback(get_fallback)
        // documentation
        .merge(
            SwaggerUi::new("/v1/api-docs/swagger-ui")
                .url("/v1/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .with_state(app_state.clone())
}
