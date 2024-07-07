use utoipa::OpenApi;

use crate::{
    documentation::api_security_addon::SecurityAddon,
    routes::{authentication, business, collection, collector, product, users},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "3rEco API Documentation",
        version = "0.1.0",
        description = "The API documentation for the 3rEco API",
        contact(
            name = "Use-IT",
            url = "https://use-it.co.za",
        ),
        license(
            name = "GPL-3.0",
        )
    ),
    paths(
        authentication::login::user,
        authentication::check::index,
        users::view::users,
        users::view::user,
        users::add::user,
        users::delete::user,
        users::update::user,
        business::view::businesses,
        business::view::business,
        business::add::business,
        business::update::business,
        business::delete::business,
        collector::view::collectors,
        collector::view::collector,
        collector::add::collector,
        collector::update::collector,
        collector::delete::collector,
        product::view::products,
        product::view::product,
        product::add::product,
        product::update::product,
        product::delete::product,
        collection::view::collections,
        collection::view::collection,
        collection::add::collection,
        collection::update::collection,
        collection::delete::collection,
    ),
    components(
        schemas(
            authentication::login::LoginPayload,
            crate::authentication::roles::Role,
            users::add::AddUserPayload,
            users::update::UpdateUserPayload,
            business::add::AddBusinessPayload,
            business::update::UpdateBusinessPayload,
            collector::add::AddCollectorPayload,
            collector::update::UpdateCollectorPayload,
            product::add::AddProductPayload,
            product::update::UpdateProductPayload,
            collection::add::AddCollectionPayload,
            collection::update::UpdateCollectionPayload,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Authentication routes."),
        (name = "Business", description = "Business routes."),
        (name = "Collector", description = "Collector routes."),
        (name = "Collection", description = "Collection routes."),
        (name = "Product", description = "Product routes."),
        (name = "Users", description = "Users routes."),
    ),
    servers(
        (
            url = "http://localhost:4000",
            description = "Development server",
        ),
        (
            url = "https://3reco.co.za/api",
            description = "Production server",
        ),
    )
)]
pub struct ApiDoc;
