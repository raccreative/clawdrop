
use crate::{errors::list::ListError, network::{get_developed_games_list}, utils::get_api_key};

pub fn run() -> Result<(), ListError> {
    let api_key = get_api_key()?;

    let response = get_developed_games_list(api_key)?;

    if response.games.is_empty() {
        println!("No games found.");
        return Ok(());
    }

    // We create a table to list our games nicely
    println!(
        "{:<10} | {:<30} | {:<15} | {:<15} | {:<15} | {:<15} | {:<15}", 
        "ID", 
        "Title", 
        "URL Identifier",
        "Windows Version", 
        "Linux Version", 
        "Mac Version", 
        "HTML Version"
    );

    println!(
        "{:-<10}-+-{:-<30}-+-{:-<15}-+-{:-<15}-+-{:-<15}-+-{:-<15}-+-{:-<15}", 
        "", "", "", "", "", "", ""
    );

    for game in response.games {
        println!(
            "{:<10} | {:<30} | {:<15} | {:<15} | {:<15} | {:<15} | {:<15}", 
            game.id, 
            game.title,
            game.url_identifier.unwrap_or("null".to_string()),
            game.windows_version.unwrap_or("null".to_string()),
            game.linux_version.unwrap_or("null".to_string()),
            game.mac_version.unwrap_or("null".to_string()),
            game.html_version.unwrap_or("null".to_string()),
        );
    }

    Ok(())
}