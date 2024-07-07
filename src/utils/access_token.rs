use anyhow::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaAuthResult {
    pub api_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaAuthResponse {
    pub result: Option<VsaAuthResult>,
}

/// ### Kaseya VSA Rest API api_token (Deprecated)
///
/// Acquires an access token from Kaseya VSA Rest API.
///
/// ### Example
///
/// ```
/// use anyhow::Error;
///
/// use api::utils::access_token::vsa_api_token;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let config: Config = Config::init();
///     let api_token: String = acquire_access_token(config).await?;
///
///     println!("{}", api_token);
///
///     Ok(())
/// }
/// ```
#[allow(unused)]
#[deprecated = "New authentication method required."]
pub async fn vsa_api_token(config: Config) -> Result<String, Error> {
    let username = config.vsa_username;
    let password = config.vsa_password;

    let client = Client::builder().http1_title_case_headers().build()?;

    let response = client
        .get("https://vsa.thusa.co.za/api/v1.0/auth")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

    let body = response.json::<VsaAuthResponse>().await?;
    let result = match body.result {
        Some(result) => Ok(result),
        None => Err(Error::msg("Failed to get result from body.")),
    }?;
    let api_token = match result.api_token {
        Some(api_token) => Ok(api_token),
        None => Err(Error::msg("Failed to get api_token from result.")),
    }?;

    Ok(api_token)
}