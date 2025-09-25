use std::fs;
use std::{env, path::PathBuf};

use self_update::cargo_crate_version;

use crate::constants::APP_NAME;
use crate::errors::api_key::ApiKeyError;
use crate::errors::common::CommonError;
use crate::errors::set::SetError;
use crate::network::{Game, get_developed_games_list};

pub fn get_config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\"))
            .join(APP_NAME)
    } else if cfg!(target_os = "macos") {
        env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join("Library")
            .join("Application Support")
            .join(APP_NAME)
    } else {
        let base = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|_| PathBuf::from("/"));
        base.join(APP_NAME)
    }
}

pub fn get_api_key() -> Result<String, ApiKeyError> {
    match std::env::var("CLAWDROP_API_KEY") {
        Ok(val) if !val.trim().is_empty() => Ok(val),
        _ => Err(ApiKeyError::MissingEnv),
    }
}

pub fn get_target_game() -> Result<Option<Game>, CommonError> {
    let target_path = get_config_path().join("target.json");

    if !target_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&target_path)?;
    let parsed = serde_json::from_str::<Game>(&content)?;

    Ok(Some(parsed))
}

pub fn set_target_game(id: String) -> Result<Game, SetError> {
    let api_key = get_api_key()?;

    let response = get_developed_games_list(api_key)?;

    // We find game by id or url slug
    let game = if let Ok(parsed_id) = id.parse::<u64>() {
        response.games.into_iter().find(|g| g.id == parsed_id)
    } else {
        response
            .games
            .into_iter()
            .find(|g| g.url_identifier.as_deref() == Some(&id))
    };

    // If the game is not present then it means that we do not have permission to handle that game (or does not exist)
    let Some(game) = game else {
        return Err(SetError::UnauthorizedToSetGame);
    };

    let serialized =
        serde_json::to_string_pretty(&game).map_err(|_| SetError::SerializationError)?;

    let path = get_config_path().join("target.json");
    let parent = path.parent().ok_or(SetError::InvalidConfigPath)?;
    std::fs::create_dir_all(parent)?;
    fs::write(path, serialized)?;

    Ok(game)
}

pub fn check_for_updates() -> Result<(), Box<dyn std::error::Error>> {
    let current_version = cargo_crate_version!();
    let release = self_update::backends::github::Update::configure()
        .repo_owner("raccreative")
        .repo_name("clawdrop")
        .bin_name("clawdrop")
        .current_version(current_version)
        .build()?
        .get_latest_release()?;

    let latest_version = release.version.trim_start_matches('v');
    let update_available = latest_version != current_version;

    if update_available {
        println!(
            "A new version {} is available! (current: {}) Run 'clawdrop upgrade' to update",
            latest_version, current_version
        );
    }

    Ok(())
}
