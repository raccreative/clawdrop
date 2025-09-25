use reqwest::blocking::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

use crate::{constants::{GAMES_LIST_URL, VERIFY_API_KEY_URL}, errors::network::NetworkError};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyApiKeyResponse {
    valid: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameListResponse {
    pub games: Vec<Game>
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: u64,
    pub title: String,
    pub url_identifier: Option<String>,
    pub windows_version: Option<String>,
    pub linux_version: Option<String>,
    pub mac_version: Option<String>,
    pub html_version: Option<String>,
}

pub fn build_client() -> Result<Client, reqwest::Error> {
    ClientBuilder::new()
        .use_rustls_tls()
        .build()
}

pub fn verify_api_key(key: &str) -> Result<bool, reqwest::Error> {
    let client = build_client()?;

    let response: VerifyApiKeyResponse = client
        .get(VERIFY_API_KEY_URL)
        .query(&[("apiKey", &key)])
        .send()?
        .json()?;

    Ok(response.valid)
}

pub fn get_developed_games_list(api_key: String) -> Result<GameListResponse, NetworkError> {
    let client = build_client()?;

    let res = match client.get(GAMES_LIST_URL)
    .header("x-api-key", api_key)
    .send()
    {
        Ok(r) => {
            match r.status() {
                reqwest::StatusCode::FORBIDDEN => return Err(NetworkError::InvalidApiKey),
                _ => r
            }
        }
        Err(e) => return Err(e.into()),
    };

    let response: GameListResponse = res.json()?;

    Ok(response)
}