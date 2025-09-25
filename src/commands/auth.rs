use std::{env, thread::sleep, time::Duration, io::Write};

use open::that as open_browser;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::constants::{AUTH_POLL_URL, AUTH_REQUEST_URL};
use crate::errors::auth::AuthError;
use crate::green;
use crate::network::{build_client, verify_api_key};
use crate::utils::get_config_path;

// API is JavaScript so we need to camelcase
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestResponse {
    poll_token: String,
    verify_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PollResponse {
    status: String,
    api_key: Option<String>,
}

pub fn run(force: bool, headless: bool, key: Option<String>) -> Result<(), AuthError> {
    if let Some(api_key) = key {
        let valid = verify_api_key(&api_key)?;

        if !valid {
            return Err(AuthError::InvalidProvidedKey);
        }

        save_api_key(&api_key)?;
        return Ok(());
    }
    
    if !force {
        if let Ok(existing) = env::var("CLAWDROP_API_KEY") {
            let key_exists = !existing.trim().is_empty();
            if key_exists && verify_api_key(&existing)? {
                println!("You are already authorized.");
                return Ok(());
            }
        }
    }

    let client = build_client()?;

    let auth_response = request_auth(&client)?;

    println!("Auth required. Open this url:\n{}", auth_response.verify_url);

    if !headless {
        open_browser(&auth_response.verify_url)?;
    }

    return poll_for_authorization(auth_response.poll_token, &client);
}

fn poll_for_authorization(poll_token: String, client: &Client) -> Result<(), AuthError> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(240);

    loop {
        if start.elapsed() > timeout {
            return Err(AuthError::RequestTimeout);
        }

        sleep(Duration::from_secs(2));

    let poll_res: PollResponse = client
        .get(AUTH_POLL_URL)
        .query(&[("pollToken", &poll_token)])
        .send()?
        .json()?;

        match poll_res.status.as_str() {
            "pending" => {
                println!("Awaiting authorization...");
                continue;
            }
            "authorized" => {
                if let Some(key) = poll_res.api_key {
                    println!("Authorized correctly {}", green!("✓"));
                    println!("Your API Key: {}", &key);
                    save_api_key(&key)?;
                    return Ok(());
                } else {
                    return Err(AuthError::MissingApiKeyInResponse);
                }
            }
            "expired" => return Err(AuthError::RequestExpired),
            "null" => return Err(AuthError::InvalidTokenResponse),
            _ => return Err(AuthError::UnknownStatus(poll_res.status)),
        }
    }
}

fn request_auth(client: &Client) -> Result<RequestResponse, AuthError> {
    let response: RequestResponse = client
        .post(AUTH_REQUEST_URL)
        .send()?
        .json()?;

    Ok(response)
}

fn save_api_key_to_file(key: &str) -> Result<(), AuthError> {
    let path = get_config_path().join(".api_key");

    let parent = path.parent().ok_or(AuthError::InvalidConfigPath)?;
    std::fs::create_dir_all(parent)?;

    let mut file = std::fs::File::create(&path)?;
    write!(file, "{}", key)?;

    println!("API Key saved at: {}", path.display());

    Ok(())
}

fn save_api_key(key: &str) -> Result<(), AuthError> {
    save_api_key_to_file(key)?;
    unsafe { env::set_var("CLAWDROP_API_KEY", key) };
    println!("{}", green!("Authorization complete."));
    Ok(())
}
