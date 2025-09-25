use crate::{cli::Cli, red, utils::check_for_updates};

pub mod auth;
pub mod doctor;
pub mod list;
pub mod logout;
pub mod post;
pub mod push;
pub mod set;
pub mod unset;
pub mod upgrade;
pub mod whereis;

pub fn dispatch(cli: Cli) {
    match cli.command {
        Some(crate::cli::Commands::Doctor) => {
            if let Err(e) = doctor::run() {
                eprintln!("Error in doctor: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Whereis) => {
            if let Err(e) = whereis::run() {
                eprintln!("Error in whereis: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Upgrade) => {
            if let Err(e) = upgrade::run() {
                eprintln!("Error in upgrade: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Auth {
            force,
            headless,
            key,
        }) => {
            if let Err(e) = auth::run(force, headless, key) {
                eprintln!("Error in auth: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Logout) => {
            if let Err(e) = logout::run() {
                eprintln!("Error in logout: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::List) => {
            if let Err(e) = list::run() {
                eprintln!("Error in list: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Set { id }) => {
            if let Err(e) = set::run(id) {
                eprintln!("Error in set: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Unset) => {
            if let Err(e) = unset::run() {
                eprintln!("Error in unset: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Post {
            id,
            title,
            body,
            cover,
            slug,
        }) => {
            if let Err(e) = check_for_updates() {
                eprintln!("Warning: failed to check for updates: {}", e);
            }

            if let Err(e) = post::run(id, title, body, cover, slug) {
                eprintln!("Error in post: {}", e);
                std::process::exit(1);
            }
        }
        Some(crate::cli::Commands::Push {
            id,
            exe,
            ignore,
            no_bump,
            os,
            path,
            shorthand,
            version,
            force,
        }) => {
            if let Err(e) = check_for_updates() {
                eprintln!("Warning: failed to check for updates: {}", e);
            }

            let args = push::PushArgs {
                id,
                os,
                exe,
                version,
                path,
                ignore,
                no_bump,
                shorthand,
                force,
            };
            // Tokio async runtime for this command
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(push::run(args)) {
                eprintln!("\n{} {}", red!("X Error in push:"), e);
                std::process::exit(1);
            }
        }
        None => {
            println!("Command not found. Use --help.");
        }
    }
}
