mod cli;
mod commands;
mod constants;
mod errors;
mod file_index;
mod macros;
mod network;
mod ui;
mod utils;

use std::{env, fs};

use clap::Parser;
use cli::Cli;

use crate::utils::get_config_path;

fn main() {
    // We get the API Key from config file if exists
    let config_path = get_config_path().join(".api_key");
    if let Ok(api_key) = fs::read_to_string(&config_path) {
        let api_key = api_key.trim();
        if !api_key.is_empty() {
            unsafe { env::set_var("CLAWDROP_API_KEY", api_key) };
        }
    }

    let cli = Cli::parse();
    commands::dispatch(cli);
}
