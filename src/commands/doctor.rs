use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use std::{fs, io, path::Path};

use crate::errors::doctor::DoctorError;
use crate::green;
use crate::network::verify_api_key;
use crate::utils::{get_api_key, get_target_game};

// To use Doctor as a reliable pipeline, we must propagate errors for exit code 1
pub fn run() -> Result<(), DoctorError> {
    check_internet()?;
    println!("Internet connection {}", green!("✓ OK"));

    println!("Checking for clawdrop authorization...");
    let key = get_api_key()?;
    println!("Api Key found: {}", mask_key(&key));

    let valid = verify_api_key(&key)?;
    if !valid {
        return Err(DoctorError::InvalidApiKey);
    }

    println!("API key valid {}", green!("✓ OK"));

    check_write_permissions(".")?;
    println!("Permissions {}", green!("✓ OK"));

    check_target_game();

    Ok(())
}

fn check_target_game() {
    println!("Checking if a target game is configured...");

    match get_target_game() {
        Ok(Some(game)) => {
            println!("Target game found: {}", green!(&game.title));
        }
        Ok(None) => {
            println!("No target game configured (you can use 'clawdrop set' to define one).");
        }
        Err(_) => {
            println!("Target file exists but could not be read or parsed.");
        }
    }
}

fn check_internet() -> Result<(), DoctorError> {
    println!("Checking internet connection...");

    let addr: SocketAddr = "8.8.8.8:53"
        .parse()
        .map_err(|e| DoctorError::InternetUnavailable {
            message: "Could not parse address".into(),
            source: Some(Box::new(e)),
        })?; // Google DNS

    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).map_err(|e| {
        DoctorError::InternetUnavailable {
            message: "Could not connect to 8.8.8.8:53 (Google DNS)".into(),
            source: Some(Box::new(e)),
        }
    })?;

    Ok(())
}

fn check_write_permissions<P: AsRef<Path>>(path: P) -> io::Result<()> {
    println!("Checking current directory permissions...");

    let test_path = path.as_ref().join(".clawdrop_write_test");
    fs::write(&test_path, b"test")?;
    fs::remove_file(test_path)?;
    Ok(())
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "*".repeat(key.len())
    } else {
        format!(
            "{}{}{}",
            &key[..4],
            "*".repeat(key.len().saturating_sub(8)),
            &key[key.len().saturating_sub(4)..]
        )
    }
}
